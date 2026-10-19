use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_oracle as oracle;

/// "Landfall \u{2014} Whenever a land you control enters, you gain 1 life." (CR 207.2c, 119.3)
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        LANDFALL,
        |g, item| effects::gain_life(g, item, 1),
        "Landfall \u{2014} Whenever a land you control enters, you gain 1 life.",
    )],
    ..oracle::EUMIDIAN_TERRABOTANIST
};
