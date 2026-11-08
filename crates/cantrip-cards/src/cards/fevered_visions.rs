use cantrip_api::*;
use cantrip_mechanics::{cards_in_hand, triggers};
use cantrip_oracle as oracle;

/// "that player draws a card. If the player is your opponent and has four or more cards in hand, this enchantment deals 2
/// damage to that player." (CR 121.1, 603.2): the draw happens first and the hand size is read after it, with no action
/// between the two steps.
fn draw_and_punish(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(player) = item.subject else { return Ok(()) };
    g.draw(player, 1);
    g.end_sentence();
    if player != item.controller && cards_in_hand(g, player) >= 4 {
        g.damage(item.object, Target::Player(player), 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::EACH_PLAYERS_END_STEP,
        draw_and_punish,
        "At the beginning of each player's end step, that player draws a card. If the player is your opponent and has four or more cards in hand, this enchantment deals 2 damage to that player.",
    )],
    ..oracle::FEVERED_VISIONS
};
