use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {C}." (CR 605.1a).
/// "{1}{U}{R}, {T}: Draw a card, then discard a card." (CR 608.2c).
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{1}{U}{R}").and(TAP),
            cantrip_mechanics::effects::loot,
            "{1}{U}{R}, {T}: Draw a card, then discard a card.",
        ),
    ],
    ..oracle::DESOLATE_LIGHTHOUSE
};
