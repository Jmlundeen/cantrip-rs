use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// Exile the resolving spell with two scream counters (CR 406.3, 122.1).
fn exile_with_screams(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for card in g.exile_all(&[item.object]) {
        if g.obj(card).zone.is(ZoneKind::Exile) {
            g.add_counters(card, CounterKind::Scream, 2);
        }
    }
    Ok(())
}

/// The intervening condition is checked both when the ability triggers and when it resolves (CR 603.4).
fn exiled_with_scream(q: &dyn Query, asker: &Asker) -> bool {
    is_live(q, asker.source)
        && q.obj(asker.source).zone.is(ZoneKind::Exile)
        && q.obj(asker.source).counters(CounterKind::Scream) > 0
}

/// Remove one counter; if none remain, put this card into its graveyard and return each player's creature cards (CR 608.2c).
fn upkeep(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.remove_counters(item.object, CounterKind::Scream, 1);
    g.end_sentence();
    if g.obj(item.object).counters(CounterKind::Scream) == 0 {
        g.put_into_graveyard(&[item.object]);
        let returning: Vec<(ObjectId, PlayerId)> = g.state().apnap().flat_map(|player| {
            g.state().zone(Zone::graveyard(player)).iter().copied()
                .filter(|&id| !g.obj(id).token && g.chars(id).is(CardType::Creature))
                .map(move |id| (id, player))
        }).collect();
        g.put_all_onto_battlefield(&returning)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], exile_with_screams),
    abilities: &[triggered(triggers::YOUR_UPKEEP, upkeep,
        "At the beginning of your upkeep, if this card is exiled with a scream counter on it, remove a scream counter from it. If there are no more scream counters on it, put it into your graveyard and each player returns all creature cards from their graveyard to the battlefield.")
        .condition(exiled_with_scream)
        .functions_in(ZoneKinds::of(&[ZoneKind::Exile]))],
    ..oracle::ALL_HALLOWS_EVE
};
