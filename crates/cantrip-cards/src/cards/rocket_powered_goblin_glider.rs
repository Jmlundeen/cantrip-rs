use cantrip_api::*;
use cantrip_mechanics::attach::{attach_to_target, attached_object, equip};
use cantrip_mechanics::statics::{have_flying_and_haste, plus_pt};
use cantrip_mechanics::{history, mayhem, targets};
use cantrip_oracle as oracle;

/// "When this Equipment enters, if it was cast from your graveyard" (CR 603.4, 603.6a, 601.2a): it entered as the spell that
/// resolved, and that spell was cast from a graveyard. A card in a graveyard is in its owner's (CR 404.1), so the graveyard was
/// yours only if you own the Glider: one cast from an opponent's graveyard (Tinybones, the Pickpocket) was not cast from yours,
/// though you cast it. The cast is the one the history holds for the spell (the history is as it was just before the entry); a
/// permanent put onto the battlefield, or a copy, was never cast from anywhere. It cannot change after it triggers, so it is not
/// asked again.
fn enters_if_cast_from_your_graveyard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, to, reason: ZoneReason::Resolve, .. }
            if *new == source
                && to.is(ZoneKind::Battlefield)
                && history::cast_from_graveyard(q.history(), *old).is_some()
                && q.obj(source).owner == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

const THIS_ENTERS_IF_CAST_FROM_YOUR_GRAVEYARD: Trigger = when(EventMask::ZONE_CHANGE, enters_if_cast_from_your_graveyard);

pub static CARD: CardDef = CardDef {
    ways: &[mayhem::way(mana("{2}"), "Mayhem {2}")],
    abilities: &[
        triggered(
            THIS_ENTERS_IF_CAST_FROM_YOUR_GRAVEYARD,
            attach_to_target,
            "When this Equipment enters, if it was cast from your graveyard, attach it to target creature you control.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL]),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<2, 0>, "Equipped creature gets +2/+0."),
        static_ability(Layer::Ability, attached_object, have_flying_and_haste, "Equipped creature has flying and haste."),
    ],
    ..oracle::ROCKET_POWERED_GOBLIN_GLIDER
};
