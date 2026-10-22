use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Whenever a player discards one or more cards" (CR 603.2c): once for each player discarding in one batch, the
/// discards of each player grouped by the player in the data; the subject is not a target.
fn player_discards(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    let Event::Discarded { player, .. } = event else { return None };
    Some(Fired { subject: Target::Player(*player), data: [0, player.0 as u32] })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::each_opponent_discards_a_card, "When Tinybones enters, each opponent discards a card."),
        triggered(
            when(EventMask::DISCARDED, player_discards),
            |g, item| {
                effects::damage_each_opponent(g, item, 1);
                Ok(())
            },
            "Whenever a player discards one or more cards, Tinybones deals 1 damage to each opponent.",
        )
        .once_per_batch()
        .batch_by_data(),
    ],
    ..oracle::TINYBONES_POCKET_NUISANCE
};
