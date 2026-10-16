use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{self, THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    kicker: multikicker(mana("{2}")),
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "Benediction of the Omnissiah \u{2014} When this creature enters, draw a card."),
        triggers::SQUAD,
    ],
    ..oracle::SICARIAN_INFILTRATOR
};
