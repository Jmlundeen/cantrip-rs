use cantrip_api::*;
use cantrip_mechanics::battlefront;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "creatures you control get +X/+X until end of turn, where X is the number of opponents you attacked this turn" (CR
/// 611.2, 611.2c): X is read as the ability resolves, and the creatures are those Korok's controller controls then.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = battlefront::opponents_attacked_this_turn(g, item.controller) as i32;
    effects::pump_creatures_you_control(g, item, x, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        grow,
        "Battlefront — Whenever Korok attacks, creatures you control get +X/+X until end of turn, where X is the number of opponents you attacked this turn.",
    )],
    ..oracle::KOROK_LIBERATED_RAGE
};
