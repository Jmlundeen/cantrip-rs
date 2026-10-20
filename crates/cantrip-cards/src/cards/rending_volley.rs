use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Rending Volley deals 4 damage to target white or blue creature." (CR 120.3.) "This spell can't be countered" is the
/// card's keyword (CR 101.2).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::creature_of_either_color::<{ Color::White as u8 }, { Color::Blue as u8 }>()], effect),
    ..oracle::RENDING_VOLLEY
};
