use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Grim Monolith doesn't untap during your untap step."
    keywords: Keywords::of(&[Keyword::DoesNotUntap]),
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 3), "{T}: Add {C}{C}{C}."),
        ability(cost("{4}"), cantrip_mechanics::effects::untap_self, "{4}: Untap Grim Monolith."),
    ],
    ..oracle::GRIM_MONOLITH
};
