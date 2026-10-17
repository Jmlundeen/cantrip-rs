use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target creature phases out until this enchantment leaves the battlefield. Tap that creature as it phases in this way."
/// (CR 702.26b, 610.4): it does not phase in during an untap step (CR 610.4a), but as soon as Oubliette has left, tapped,
/// which is no becoming tapped (the rulings). If Oubliette has left before this resolves, the creature does not phase out
/// (CR 610.4b; the ruling). Auras and Equipment attached to it phase out with it (CR 702.26g).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.phase_out_until_leaves(creature, item.object, true);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effect, "When this enchantment enters, target creature phases out until this enchantment leaves the battlefield. Tap that creature as it phases in this way.")
        .targets(&[targets::CREATURE])],
    ..oracle::OUBLIETTE
};
