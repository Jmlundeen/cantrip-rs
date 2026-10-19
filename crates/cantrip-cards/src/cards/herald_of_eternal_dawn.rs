use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "You can't lose the game and your opponents can't win the game." (CR 104.3, 104.2b). Flash and flying are core keywords.
pub static CARD: CardDef = CardDef {
    loss_rules: &[statics::you_cant_lose_the_game],
    win_rules: &[statics::your_opponents_cant_win_the_game],
    ..oracle::HERALD_OF_ETERNAL_DAWN
};
