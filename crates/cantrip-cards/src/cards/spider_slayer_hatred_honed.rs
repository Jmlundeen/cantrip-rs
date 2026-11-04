use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Whenever Spider-Slayer deals damage to a Spider" (CR 603.2): any damage, combat or not (the history records damage only when
/// more than 0 is dealt, CR 120.8), dealt to a permanent with the Spider creature type, the word read through this card's own
/// text (CR 612). The Spider is the trigger's subject.
fn deals_damage_to_a_spider(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Object(id), .. }
            if *from == source && q.chars(*id).has_subtype(q.words(source).subtype(Subtype::Spider)) =>
        {
            Fired::about(Target::Object(*id))
        }
        _ => None,
    }
}

const DEALS_DAMAGE_TO_A_SPIDER: Trigger = when(EventMask::DAMAGE, deals_damage_to_a_spider);

/// "... destroy that creature": the Spider it dealt damage to, the trigger's subject; nothing if it has left the battlefield
/// since (CR 400.7). It is a triggered ability, not deathtouch: it destroys a Spider that survived the damage.
fn destroy_that_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject
        && targets::permanent(g, creature)
    {
        g.destroy(creature);
    }
    Ok(())
}

/// "{6}, Exile this card from your graveyard: Create two tapped 1/1 colorless Robot artifact creature tokens with flying"
/// (CR 111.1): both tokens made together, tapped (CR 701.7).
fn make_robots(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let robot = g.card_id(&tokens::ROBOT_1_1_FLYING);
    g.create_tokens(robot, item.controller, 2, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(DEALS_DAMAGE_TO_A_SPIDER, destroy_that_creature, "Whenever Spider-Slayer deals damage to a Spider, destroy that creature."),
        ability(
            cost("{6}").and(EXILE_THIS),
            make_robots,
            "{6}, Exile this card from your graveyard: Create two tapped 1/1 colorless Robot artifact creature tokens with flying.",
        )
        .from(ZoneKind::Graveyard),
    ],
    ..oracle::SPIDER_SLAYER_HATRED_HONED
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROBOT_1_1_FLYING];
