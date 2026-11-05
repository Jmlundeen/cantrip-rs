use cantrip_api::*;
use cantrip_mechanics::{effects, statics};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, statics::itself_during_your_turn, statics::have_first_strike, "During your turn, this creature has first strike.")],
    abilities: &[ability(cost("{4}{W}"), effects::creatures_you_control_get::<1, 1>, "{4}{W}: Creatures you control get +1/+1 until end of turn.")],
    ..oracle::BEARER_OF_GLORY
};
