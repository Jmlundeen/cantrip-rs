use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "This artifact doesn't untap during your untap step."
    keywords: Keywords::of(&[Keyword::DoesNotUntap]),
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 3), "{T}: Add {C}{C}{C}."),
        ability(cost("{3}"), cantrip_mechanics::effects::untap_self, "{3}: Untap this artifact."),
    ],
    ..oracle::BASALT_MONOLITH
};
