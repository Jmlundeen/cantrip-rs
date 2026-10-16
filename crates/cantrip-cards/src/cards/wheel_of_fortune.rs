use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { spell: spell(&[], effects::each_player_discards_hand_then_draws_seven), ..oracle::WHEEL_OF_FORTUNE };
