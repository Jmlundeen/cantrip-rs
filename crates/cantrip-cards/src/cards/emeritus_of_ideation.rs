use cantrip_api::*;
use cantrip_mechanics::{choose, triggers::ward, triggers, effects};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Exile", "exile eight cards from your graveyard?", "If you do, it becomes prepared", "Exile eight", "Don't exile");

/// "You may exile eight cards from your graveyard. If you do, this creature becomes prepared." A graveyard short of
/// eight is never asked: the payment is impossible, so it was not made.
fn exile_eight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    if graveyard.len() < 8 {
        return Ok(());
    }
    if !effects::ask_may(g, you, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    let chosen = choose::cards(g, you, item.object, Purpose::Exile, graveyard, 8, false)?;
    g.exile_all(&chosen);
    g.become_prepared(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)"),
        triggered(triggers::THIS_ATTACKS, exile_eight, "Whenever this creature attacks, you may exile eight cards from your graveyard. If you do, this creature becomes prepared."),
    ],
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&crate::ancestral_recall::CARD),
    ..oracle::EMERITUS_OF_IDEATION
};
