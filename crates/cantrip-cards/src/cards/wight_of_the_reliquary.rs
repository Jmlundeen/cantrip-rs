use cantrip_api::*;
use cantrip_mechanics::statics::{itself, plus_one_per_card_type_in_your_graveyard};
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, itself, plus_one_per_card_type_in_your_graveyard::<{ CardType::Creature as u8 }>, "This creature gets +1/+1 for each creature card in your graveyard.")],
    abilities: &[ability(cost("{T}").and(sacrifice(1, targets::filters::another_creature)), library::search_for_a_land_onto_the_battlefield_tapped, "{T}, Sacrifice another creature: Search your library for a land card, put it onto the battlefield tapped, then shuffle.")],
    ..oracle::WIGHT_OF_THE_RELIQUARY
};
