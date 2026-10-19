use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "You can't lose the game and your opponents can't win the game." A draw is no loss (the 2004-12-01 ruling).
pub static CARD: CardDef = CardDef {
    loss_rules: &[statics::you_cant_lose_the_game],
    win_rules: &[statics::your_opponents_cant_win_the_game],
    ..oracle::PLATINUM_ANGEL
};
