use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Defibrillating Current deals 4 damage to target creature or planeswalker and you gain 2 life." With its one target illegal
/// it does not resolve at all, so the life is not gained (CR 608.2b).
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], |g, item| effects::damage_target_and_gain_life(g, item, 4, 2)),
    ..oracle::DEFIBRILLATING_CURRENT
};
