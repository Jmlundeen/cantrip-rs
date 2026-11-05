use cantrip_api::*;
use cantrip_mechanics::triggers::{ward, YOUR_UPKEEP};
use cantrip_mechanics::{choose, effects, select};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Exile", "exile two cards from your graveyard?", "If you do, put a +1/+1 counter on this creature", "Exile two", "Don't exile");

/// "You may exile two cards from your graveyard. If you do, put a +1/+1 counter on this creature." A graveyard short of two is
/// never asked: the payment is impossible, so it was not made (CR 608.2d).
fn exile_two_for_a_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    if graveyard.len() < 2 || !effects::ask_may(g, you, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    let chosen = choose::select(g, you, item.object, Purpose::Exile, graveyard, select::cards(select::exactly(2)))?;
    g.exile_all(&chosen);
    g.end_sentence();
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)"),
        triggered(YOUR_UPKEEP, exile_two_for_a_counter, "At the beginning of your upkeep, you may exile two cards from your graveyard. If you do, put a +1/+1 counter on this creature."),
    ],
    ..oracle::AEGIS_SCULPTOR
};
