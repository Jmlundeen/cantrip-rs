use cantrip_api::*;
use cantrip_mechanics::{delayed, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of combat on your turn, this creature gains banding until end of combat." (CR 608.2, 611.2a).
fn creature_gains_banding_combat(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::Banding])), Duration::EndOfCombat);
    Ok(())
}

/// "Whenever this creature becomes blocked by a Wall" (CR 603.2).
fn when_creature_becomes_blocked_by_wall(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::Blocks { blocker, attacker, .. } = event else { return None };
    let id = *attacker;
    let other = *blocker;
    let subject = Target::Object(other);
    if asker.source == id
        && q.chars(other).has_subtype(q.words(asker.source).subtype(Subtype::Wall))
    {
        return Fired::about(subject);
    }
    None
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::BEGINNING_OF_YOUR_COMBAT,
            creature_gains_banding_combat,
            "At the beginning of combat on your turn, this creature gains banding until end of combat.",
        ),
        triggered(
            when_asked(EventMask::BLOCKS, when_creature_becomes_blocked_by_wall),
            delayed::destroy_that_creature_at_end_of_combat,
            "Whenever this creature becomes blocked by a Wall, destroy that Wall at end of combat.",
        ),
    ],
    delayed: &[delayed::destroy_that_at_end_of_combat("Destroy that Wall at end of combat.")],
    ..oracle::BATTERING_RAM
};
