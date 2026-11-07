use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "When this creature enters, mill two cards, then put a +1/+1 counter on this creature for each artifact and/or creature card in your graveyard." (CR 608.2, 122.1, 701.17a).
fn mill_two_cards_put_1(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mills_0: Vec<(PlayerId, u32)> = Some(item.controller)
        .into_iter()
        .map(|p| (p, 2u32))
        .collect();
    g.mill_together(&mills_0);
    g.end_sentence();
    let mut counters_1: Vec<(ObjectId, CounterKind, i32)> = Vec::new();
    let amount = {
        let q: &dyn Query = g;
        q.state()
            .zone(Zone::graveyard(item.controller))
            .iter()
            .filter(|&&id| q.chars(id).is(CardType::Artifact) || q.chars(id).is(CardType::Creature))
            .count() as i32
    };
    for t in Some(item.object).into_iter().map(Target::Object).collect::<Vec<Target>>() {
        if let Target::Object(object) = t {
            counters_1.push((object, CounterKind::PlusOne, amount));
        }
    }
    g.add_counters_together(&counters_1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            mill_two_cards_put_1,
            "When this creature enters, mill two cards, then put a +1/+1 counter on this creature for each artifact and/or creature card in your graveyard.",
        ),
    ],
    ..oracle::CETI_EEL
};
