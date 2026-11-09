use cantrip_api::*;
use cantrip_mechanics::{choose, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of the chosen player's upkeep, this artifact deals X damage to that player, where X is the number of cards in their hand minus 4." (CR 608.2, 120.3).
fn artifact_deals_x_damage_player(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let amount_1 = {
        let q: &dyn Query = g;
        (item
            .subject
            .player()
            .into_iter()
            .flat_map(|p: PlayerId| q.state().zone(Zone::hand(p)).to_vec())
            .collect::<Vec<ObjectId>>()
            .into_iter()
            .collect::<Vec<ObjectId>>()
            .len() as i64 - 4)
            .max(0) as u32
    };
    let recipients_1 = item
        .subject
        .player()
        .into_iter()
        .map(Target::Player)
        .collect::<Vec<Target>>();
    let batch: Vec<(ObjectId, Target, u32)> = recipients_1
        .iter()
        .map(|t| (item.object, *t, amount_1))
        .collect();
    g.damage_together(&batch);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::CHOSEN_PLAYERS_UPKEEP,
            artifact_deals_x_damage_player,
            "At the beginning of the chosen player's upkeep, this artifact deals X damage to that player, where X is the number of cards in their hand minus 4.",
        ),
    ],
    enters: Some(choose::opponent_as_enters_to_take_upkeep_damage),
    ..oracle::BLACK_VISE
};
