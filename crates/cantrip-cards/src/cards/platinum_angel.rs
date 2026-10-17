use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You can't lose the game" (CR 104.3): however its controller would lose - 0 or less life, a draw from an empty library,
/// ten poison counters, an effect that says so - they don't, while this is on the battlefield with its abilities. A
/// concession is no effect and still loses (CR 104.3a), and a draw is no loss (the 2004-12-01 ruling).
fn you_cant_lose(_: &dyn Query, asker: &Asker, player: PlayerId, _: LossReason) -> bool {
    player == asker.controller
}

/// "Your opponents can't win the game" (CR 104.2b, 102.2): an effect that says an opponent wins does nothing.
fn your_opponents_cant_win(_: &dyn Query, asker: &Asker, player: PlayerId) -> bool {
    player != asker.controller
}

pub static CARD: CardDef = CardDef {
    loss_rules: &[you_cant_lose],
    win_rules: &[your_opponents_cant_win],
    ..oracle::PLATINUM_ANGEL
};
