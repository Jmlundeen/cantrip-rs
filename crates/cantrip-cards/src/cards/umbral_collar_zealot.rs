use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

/// "Surveil 1." (CR 701.25.)
fn surveil_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        sacrifice(1, filters::another_creature_or_artifact),
        surveil_one,
        "Sacrifice another creature or artifact: Surveil 1.",
    )],
    ..oracle::UMBRAL_COLLAR_ZEALOT
};
