use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "You may play lands from your graveyard" (CR 305.1): each still uses a land play (CR 305.2).
    play_rules: &[PlayRule::LandsFromGraveyard],
    ..oracle::CRUCIBLE_OF_WORLDS
};
