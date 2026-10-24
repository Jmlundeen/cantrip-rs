use cantrip_api::*;
use cantrip_mechanics::{pump, statics};
use cantrip_oracle as oracle;

/// "{1}{R}: This creature gets +1/+0 until end of turn." (CR 611.2)
fn pump_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}{R}"), pump_self, "{1}{R}: This creature gets +1/+0 until end of turn.")],
    statics: &[static_ability(Layer::Ability, statics::itself_during_your_turn, statics::have_first_strike, "During your turn, this creature has first strike.")],
    ..oracle::SKILLED_BATTLECARVER
};
