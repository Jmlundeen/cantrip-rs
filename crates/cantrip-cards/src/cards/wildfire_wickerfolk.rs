use cantrip_api::*;
use cantrip_mechanics::statics::{have_trample, itself_with_delirium, plus_pt};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::PtModify, itself_with_delirium, plus_pt::<1, 1>, "Delirium \u{2014} This creature gets +1/+1 as long as there are four or more card types among cards in your graveyard."),
        static_ability(Layer::Ability, itself_with_delirium, have_trample, "Delirium \u{2014} This creature has trample as long as there are four or more card types among cards in your graveyard."),
    ],
    ..oracle::WILDFIRE_WICKERFOLK
};
