use cantrip_api::*;
use cantrip_mechanics::targets::filters::nonland;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS;
use cantrip_mechanics::{choose, choose_from_hand};
use cantrip_oracle as oracle;

/// The choices of each opponent, as `labels` below.
const SACRIFICE: u8 = 0;
const DISCARD: u8 = 1;
const LOSE: u8 = 2;

/// "Put a blight counter on it. Then for each blight counter on it, each opponent loses 4 life unless that player sacrifices
/// a nonland permanent of their choice or discards a card." The counter goes on it only while it is on the battlefield
/// (`add_counters` puts none on an object that has left, CR 400.7); the counters are counted on it, or as it last existed
/// there (CR 608.2h). For each, the opponents choose in turn order (CR 101.4) - to sacrifice a nonland permanent, to discard
/// a card, or to lose 4 life, which is the one choice left to a player with neither - and then the sacrifices, the discards
/// and the losses of life happen together; then again for the next counter.
fn blight_and_punish(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Blight, 1);
    let counters = g.obj(item.object).counters(CounterKind::Blight).max(0);
    let card = g.card_id(&CARD);
    for _ in 0..counters {
        let (mut sacrificed, mut discarded, mut losers) = (Vec::new(), Vec::new(), Vec::new());
        for opponent in g.state().opponents(item.controller).collect::<Vec<_>>() {
            let asker = Asker { source: item.object, controller: opponent };
            let permanents: Vec<ObjectId> = g.permanents_of(opponent).into_iter().filter(|&id| nonland(g, &asker, id)).collect();
            let mut options = Vec::new();
            if !permanents.is_empty() {
                options.push(SACRIFICE);
            }
            if !g.state().zone(Zone::hand(opponent)).is_empty() {
                options.push(DISCARD);
            }
            options.push(LOSE);
            let choice = if options.len() == 1 {
                LOSE
            } else {
                let labels = options.iter().map(|&i| Label::Card(card, i)).collect();
                options[choose::one_of(g, opponent, item.object, labels)? as usize]
            };
            match choice {
                SACRIFICE => sacrificed.extend(choose::cards(g, opponent, item.object, Purpose::Sacrifice, permanents, 1, false)?),
                DISCARD => discarded.extend(choose_from_hand(g, opponent, item.object, 1)?),
                _ => losers.push(opponent),
            }
        }
        g.sacrifice(&sacrificed);
        g.discard_together(&discarded);
        for player in losers {
            g.lose_life(player, 4);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "As an additional cost to cast this spell, you may sacrifice any number of nonland permanents. This spell costs {1}
    // less to cast for each permanent sacrificed this way" (CR 601.2b, 601.2f, 118.7a): how many is announced, the total
    // cost determined with it, and which ones chosen as it is paid.
    additional_cost: Some(Cost { variable: Some(VariableCost::SacrificeAnyNumber { filter: nonland, less_each: 1 }), ..Cost::NONE }),
    abilities: &[triggered(THIS_ENTERS_OR_ATTACKS, blight_and_punish, "Whenever this creature enters or attacks, put a blight counter on it. Then for each blight counter on it, each opponent loses 4 life unless that player sacrifices a nonland permanent of their choice or discards a card.")],
    labels: &["Sacrifice a nonland permanent", "Discard a card", "Lose 4 life"],
    ..oracle::ROTTENMOUTH_VIPER
};
