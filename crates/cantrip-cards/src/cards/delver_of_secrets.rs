use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::top_of_library;
use cantrip_mechanics::triggers::YOUR_UPKEEP;
use cantrip_oracle as oracle;

const REVEAL: Question = Question::new(
    "RevealTopToTransform",
    "reveal the top card of your library?",
    "If it's an instant or sorcery card, this creature transforms. The card stays on top either way",
    "Reveal it",
    "Keep it hidden",
);

/// "At the beginning of your upkeep, look at the top card of your library. You may reveal that card. If an instant or
/// sorcery card is revealed this way, transform this creature." The card is looked at and may be revealed whatever it is
/// (the 2011-09-22 ruling); it stays on top either way. "Transform this creature" is the ability's source, which does
/// nothing if it is no longer that object (CR 400.7), has transformed since the ability went on the stack (CR 701.27f), or
/// is not a double-faced card - a Clone copying Delver (CR 712.9). An empty library has nothing to look at or reveal.
fn look_and_reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let Some(&top) = top_of_library(g, you, 1).first() else {
        return Ok(());
    };
    g.look_at(you, &[top]);
    if !ask_may(g, you, item.object, Purpose::Ask(&REVEAL))? {
        return Ok(());
    }
    g.reveal(you, &[top]);
    let card = g.chars(top);
    if card.is(CardType::Instant) || card.is(CardType::Sorcery) {
        g.transform_by(item.object, item)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_UPKEEP,
        look_and_reveal,
        "At the beginning of your upkeep, look at the top card of your library. You may reveal that card. If an instant or sorcery card is revealed this way, transform this creature.",
    )],
    ..oracle::DELVER_OF_SECRETS
};
