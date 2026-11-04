use cantrip_api::*;
use cantrip_mechanics::{flashback, tokens};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[], tokens::create_a_landfall_bird),
    ways: &[flashback::way(cost("{6}{G}"), "Flashback {6}{G} (You may cast this card from your graveyard for its flashback cost. Then exile it.)")],
    ..oracle::GYSAHL_GREENS
};

pub static EXTRA: &[&CardDef] = &[&tokens::LANDFALL_BIRD];
