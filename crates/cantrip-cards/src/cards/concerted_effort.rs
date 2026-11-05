use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// The abilities Concerted Effort hands out, in the order its text lists them: "flying ... fear, first strike, double
/// strike, landwalk, protection, trample, and vigilance." Each landwalk is an ability of its own, and protection is granted
/// by its colors beside the keywords.
const KEYWORDS: &[Keyword] = &[
    Keyword::Flying,
    Keyword::Fear,
    Keyword::FirstStrike,
    Keyword::DoubleStrike,
    Keyword::Islandwalk,
    Keyword::Plainswalk,
    Keyword::Swampwalk,
    Keyword::Mountainwalk,
    Keyword::Forestwalk,
    Keyword::NonbasicLandwalk,
    Keyword::LegendaryLandwalk,
    Keyword::Trample,
    Keyword::Vigilance,
];

/// "At the beginning of each upkeep, creatures you control gain [each ability] until end of turn if a creature you control
/// has [it]" (CR 503.1a, 611.2a): read as the ability resolves, once for each player's upkeep, whoever's turn it is.
fn grant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::creatures_you_control_gain_keywords_they_have(g, item, KEYWORDS);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::EACH_UPKEEP,
        grant,
        "At the beginning of each upkeep, creatures you control gain flying until end of turn if a creature you control has flying. The same is true for fear, first strike, double strike, landwalk, protection, trample, and vigilance.",
    )],
    ..oracle::CONCERTED_EFFORT
};
