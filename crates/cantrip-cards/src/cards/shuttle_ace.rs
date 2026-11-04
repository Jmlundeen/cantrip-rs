use cantrip_api::*;
use cantrip_mechanics::{effects, statics, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_BECOMES_TAPPED,
        |g, item| effects::gain_life(g, item, 2),
        "Whenever this creature becomes tapped, you gain 2 life.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        statics::itself_during_your_turn,
        statics::have_flying,
        "During your turn, this creature has flying.",
    )],
    ..oracle::SHUTTLE_ACE
};
