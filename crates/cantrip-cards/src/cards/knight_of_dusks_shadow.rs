use cantrip_api::*;
use cantrip_mechanics::pump;
use cantrip_mechanics::statics::opponents_cant_gain_life;
use cantrip_oracle as oracle;

/// "This creature gets +1/+1 until end of turn" (CR 611.2).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cant_gain_life: &[opponents_cant_gain_life],
    abilities: &[ability(cost("{1}{B}"), grow, "{1}{B}: This creature gets +1/+1 until end of turn.")],
    ..oracle::KNIGHT_OF_DUSKS_SHADOW
};
