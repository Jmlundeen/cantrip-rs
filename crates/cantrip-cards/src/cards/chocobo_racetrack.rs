use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        LANDFALL,
        tokens::create_a_landfall_bird,
        "Landfall \u{2014} Whenever a land you control enters, create a 2/2 green Bird creature token with \"Whenever a land you control enters, this token gets +1/+0 until end of turn.\"",
    )],
    ..oracle::CHOCOBO_RACETRACK
};

pub static EXTRA: &[&CardDef] = &[&tokens::LANDFALL_BIRD];
