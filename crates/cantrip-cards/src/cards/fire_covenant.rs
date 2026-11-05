use cantrip_api::*;
use cantrip_mechanics::{damage_divided, targets};
use cantrip_oracle as oracle;

/// Fire Covenant deals X damage divided as you choose among any number of target creatures (CR 601.2d), X being the
/// life paid as an additional cost: any number includes zero (CR 107.1c); each chosen target takes its own share, and one
/// that has since become illegal is left out with it.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_divided(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE.between(0, TargetSpec::COUNTED)], effect),
    divide: Some(Divide { total: TargetSpec::COUNTED, reads: None }),
    // As an additional cost to cast this spell, pay X life.
    additional_cost: Some(Cost { pay_x_life: true, ..Cost::NONE }),
    ..oracle::FIRE_COVENANT
};
