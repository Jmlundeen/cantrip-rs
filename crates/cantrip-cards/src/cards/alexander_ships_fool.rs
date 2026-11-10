use cantrip_api::*;
use cantrip_mechanics::parley::parley;
use cantrip_oracle as oracle;

/// "{T}: Each player reveals the top card of their library, then draws a card. Alexander deals X damage to a random
/// opponent, where X is the greatest mana value among cards revealed this way." The reveal comes first, each card seen
/// by every player, and the draws follow as one instruction in APNAP order (CR 701.20, 121.2c: `parley`). X is the
/// greatest mana value among the cards revealed (CR 202.3), 0 when none was revealed; one opponent still in the game
/// is chosen at random; the damage is this creature's (CR 120.2, 113.7a).
fn reveal_then_draw_and_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let revealed = parley(g);
    let x = revealed.iter().map(|&card| g.mana_value(card)).max().unwrap_or_default();
    // A player whose opponents have all left the game has won (CR 104.2a), so a resolving ability's controller has an
    // opponent still in the game.
    let opponents: Vec<PlayerId> = g.state().opponents(item.controller).collect();
    let opponent = opponents[g.choose_at_random(opponents.len())];
    g.damage(item.object, Target::Player(opponent), x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        reveal_then_draw_and_damage,
        "{T}: Each player reveals the top card of their library, then draws a card. Alexander deals X damage to a random opponent, where X is the greatest mana value among cards revealed this way.",
    )],
    ..oracle::ALEXANDER_SHIPS_FOOL
};
