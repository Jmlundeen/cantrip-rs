use cantrip_api::*;
use cantrip_mechanics::damage_target;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    ..oracle::SEAR
};
