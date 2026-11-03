use cantrip_api::*;
use cantrip_mechanics::{targets, damage_divided};
use cantrip_oracle as oracle;

/// "Pyrotechnics deals 4 damage divided as you choose among any number of targets." (CR 608.2).
fn pyrotechnics_deals_4_damage_divided(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    damage_divided(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::ANY.between(0, TargetSpec::ANY_NUMBER)],
        pyrotechnics_deals_4_damage_divided,
    ),
    divide: Some(Divide { total: 4, of_x: None }),
    ..oracle::PYROTECHNICS
};
