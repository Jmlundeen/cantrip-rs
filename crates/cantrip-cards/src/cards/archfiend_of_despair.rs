use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::statics::opponents_cant_gain_life;
use cantrip_mechanics::triggers::EACH_END_STEP;
use cantrip_oracle as oracle;

/// "At the beginning of each end step, each opponent loses life equal to the life that player lost this turn." (CR 119.3, 603.2b, 608.2h).
fn each_opponent_loses_again(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::each_opponent_loses_life_lost_this_turn(g, item.controller);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cant_gain_life: &[opponents_cant_gain_life],
    abilities: &[triggered(
        EACH_END_STEP,
        each_opponent_loses_again,
        "At the beginning of each end step, each opponent loses life equal to the life that player lost this turn. (Damage causes loss of life.)",
    )],
    ..oracle::ARCHFIEND_OF_DESPAIR
};
