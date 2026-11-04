use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::Trample, Keyword::DoesNotUntap]),
    abilities: &[
        ability(
                cost("{9}"),
                cantrip_mechanics::effects::untap_self,
                "{9}: Untap this creature. Activate only during your upkeep.",
            )
            .condition(cantrip_mechanics::your_upkeep),
    ],
    ..oracle::COLOSSUS_OF_SARDIA
};
