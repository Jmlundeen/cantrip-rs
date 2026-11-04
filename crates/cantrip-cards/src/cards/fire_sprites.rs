use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{G}, {T}: Add {R}." is a mana ability (CR 605.1a); flying is the content file's core keyword (CR 702.9).
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(cost("{G}").and(TAP), add(Mana::R), "{G}, {T}: Add {R}.")],
    ..oracle::FIRE_SPRITES
};
