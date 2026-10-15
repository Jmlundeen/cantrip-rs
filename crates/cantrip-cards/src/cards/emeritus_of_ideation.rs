use cantrip_api::*;
use cantrip_mechanics::{choose, triggers::ward, triggers, effects};
use cantrip_oracle as oracle;

/// "This creature enters prepared" (CR 614.1d, 722.3a).
fn enters_prepared(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.prepared = true;
    Ok(())
}

/// "You may exile eight cards from your graveyard. If you do, this creature becomes prepared." A graveyard short of
/// eight is never asked: the payment is impossible, so it was not made.
fn exile_eight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    if graveyard.len() < 8 {
        return Ok(());
    }
    if !effects::ask_may(g, you, item.object, Purpose::Optional)? {
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
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature attacks, you may exile eight cards from your graveyard. If you do, this creature becomes prepared.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks,
            effect: exile_eight,
            ..TriggeredDef::BLANK
        }),
    ],
    enters: Some(enters_prepared),
    prepare: Some(&crate::ancestral_recall::CARD),
    ..oracle::EMERITUS_OF_IDEATION
};
