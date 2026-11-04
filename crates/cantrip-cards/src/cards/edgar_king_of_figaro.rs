use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Two-Headed Coin - The first time you flip one or more coins each turn, those coins come up heads and you win those
/// flips" (CR 705.3): the coins its controller flips together, when they have flipped none yet this turn - every coin of a
/// "flip five coins" (the ruling), whatever the coins show (the ruling), and a flip that cares only about heads is won too
/// (the ruling). The engine asks it before the set is flipped, so the history does not hold that set yet.
fn two_headed_coin(q: &dyn Query, me: &Asker, player: PlayerId) -> bool {
    player == me.controller
        && !q.history().this_turn().iter().any(|r| matches!(r.event, Event::CoinFlipped { player: p, .. } if p == player))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::draw_a_card_for_each_artifact_you_control, "When Edgar enters, draw a card for each artifact you control.")],
    coin_flip_rules: &[two_headed_coin],
    ..oracle::EDGAR_KING_OF_FIGARO
};
