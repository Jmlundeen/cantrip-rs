use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Each player discards their hand, then draws seven cards." Miracle {1}{R} (CR 702.94a) is the core's: the reveal as it
/// is drawn and the linked triggered ability.
pub static CARD: CardDef = CardDef {
    spell: spell(&[], effects::each_player_discards_hand_then_draws_seven),
    miracle: Some(mana("{1}{R}")),
    ..oracle::REFORGE_THE_SOUL
};
