use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;
use cantrip_mechanics::{damage_target, targets};

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    ways: &[flashback::way(cost("{4}{R}"), "Flashback {4}{R}")],
    ..oracle::FIREBOLT
};
