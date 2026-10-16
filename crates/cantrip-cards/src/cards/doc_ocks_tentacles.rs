use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects::ask_may;
use cantrip_oracle as oracle;

/// "Whenever a creature you control with mana value 5 or greater enters" (CR 603.6a): the creature under your control as it
/// entered, its mana value judged as it is on the battlefield (X is 0, CR 107.3; a token copy has the copied card's,
/// CR 707.2). The entering creature is the trigger's subject, "it".
fn big_creature_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).is(CardType::Creature)
                && q.mana_value(*new) >= 5 =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "You may attach this Equipment to it" (CR 701.3a): chosen as the ability resolves. Nothing is offered when the Equipment
/// has left the battlefield (a new object, CR 400.7) or the creature has, or is no longer a creature (CR 301.5c, 701.3b).
fn may_attach_to_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let equipment = item.object;
    let Target::Object(creature) = item.subject else { return Ok(()) };
    if !is_live(g, equipment)
        || !g.is_on_battlefield(equipment)
        || !is_live(g, creature)
        || !g.is_on_battlefield(creature)
        || !g.chars(creature).is(CardType::Creature)
    {
        return Ok(());
    }
    if ask_may(g, item.controller, equipment, Purpose::Attach)? {
        g.attach(equipment, creature);
    }
    Ok(())
}

fn plus_four_plus_four(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 4);
    chars.toughness = chars.toughness.map(|v| v + 4);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, big_creature_you_control_enters),
            may_attach_to_it,
            "Whenever a creature you control with mana value 5 or greater enters, you may attach this Equipment to it.",
        ),
        equip(mana("{5}"), "Equip {5}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_four_plus_four, "Equipped creature gets +4/+4.")],
    ..oracle::DOC_OCKS_TENTACLES
};
