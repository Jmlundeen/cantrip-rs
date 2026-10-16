use cantrip_api::*;
use cantrip_mechanics::attach::{self, attached_object, equip};
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

/// "You may attach this Equipment to it" (CR 701.3a): the entering creature is the trigger's subject. An Equipment already
/// attached elsewhere moves to it (CR 701.3c).
fn may_attach_to_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    attach::may_attach_to(g, item.controller, item.object, creature, Purpose::Attach)
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
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<4, 4>, "Equipped creature gets +4/+4.")],
    ..oracle::DOC_OCKS_TENTACLES
};
