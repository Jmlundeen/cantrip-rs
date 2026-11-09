use cantrip_api::*;
use cantrip_mechanics::commander;
use cantrip_oracle as oracle;

/// "{T}: Add one mana of any color in your commander's color identity." (CR 605.1a, 903.4f).
pub static CARD: CardDef = CardDef {
    abilities: &[commander::MANA],
    ..oracle::ARCANE_SIGNET
};
