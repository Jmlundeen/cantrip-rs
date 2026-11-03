use cantrip_api::*;
use cantrip_mechanics::{cards_in_hand, choose, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of the chosen player's upkeep, this artifact deals X damage to that player, where X is 3 minus the number of cards in their hand." (CR 608.2, 120.3).
/// With three or more cards in hand X is negative, which is no damage (CR 107.1b).
fn artifact_deals_x_damage_player(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.subject.player() else { return Ok(()) };
    let in_hand = cards_in_hand(g, player) as u32;
    let amount = if in_hand < 3 { 3 - in_hand } else { 0 };
    g.damage_together(&[(item.object, Target::Player(player), amount)]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::CHOSEN_PLAYERS_UPKEEP,
        artifact_deals_x_damage_player,
        "At the beginning of the chosen player's upkeep, this artifact deals X damage to that player, where X is 3 minus the number of cards in their hand.",
    )],
    enters: Some(choose::opponent_as_enters_to_take_upkeep_damage),
    ..oracle::THE_RACK
};
