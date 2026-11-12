use cantrip_api::*;
use cantrip_mechanics::{damage_target, permanents_with_subtype_you_control, targets};
use cantrip_oracle as oracle;

/// "Invasive Maneuvers deals 3 damage to target creature. It deals 5 damage instead if you control a Spacecraft." Whether
/// you control a Spacecraft is checked as the spell resolves (the rulings, CR 608.2c), through the layers (CR 613): a
/// permanent of yours with the Spacecraft subtype, an opponent's not counting.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spacecraft = permanents_with_subtype_you_control(g, item.controller, Subtype::Spacecraft) > 0;
    damage_target(g, item, 0, if spacecraft { 5 } else { 3 });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::INVASIVE_MANEUVERS };
