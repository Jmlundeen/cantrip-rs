use cantrip_api::*;
use cantrip_mechanics::triggers::this_enters;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "It deals 4 damage to target creature": the ability's source is the Kavu, and the target is mandatory, so the Kavu
/// itself is the target when it is the only creature.
fn bolt(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature enters, it deals 4 damage to target creature.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[targets::CREATURE],
        effect: bolt,
        ..TriggeredDef::BLANK
    })],
    ..oracle::FLAMETONGUE_KAVU
};
