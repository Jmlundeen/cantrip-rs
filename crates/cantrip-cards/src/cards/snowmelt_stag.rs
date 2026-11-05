use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::statics::{base_pt, itself_during_your_turn};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{5}{U}{U}"), effects::this_cant_be_blocked_this_turn, "{5}{U}{U}: This creature can't be blocked this turn.")],
    statics: &[static_ability(Layer::PtSet, itself_during_your_turn, base_pt::<5, 2>, "During your turn, this creature has base power and toughness 5/2.")],
    ..oracle::SNOWMELT_STAG
};
