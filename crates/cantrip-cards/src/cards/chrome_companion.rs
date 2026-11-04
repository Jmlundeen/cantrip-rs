use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_BECOMES_TAPPED;
use cantrip_mechanics::{effects, library, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_BECOMES_TAPPED, |g, item| effects::gain_life(g, item, 1), "Whenever this creature becomes tapped, you gain 1 life."),
        ability(cost("{2}, {T}"), library::put_first_target_on_the_bottom, "{2}, {T}: Put target card from a graveyard on the bottom of its owner's library.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD]),
    ],
    ..oracle::CHROME_COMPANION
};
