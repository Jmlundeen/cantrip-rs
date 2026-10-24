use cantrip_api::*;
use cantrip_mechanics::library::{find_to_battlefield, object_names};
use cantrip_mechanics::select::{self, part, up_to};
use cantrip_mechanics::{choose, effects};
use cantrip_oracle as oracle;

const EXILE_AND_SEARCH: Question = Question::new(
    "Exile",
    "exile this spell and four cards named Sphinx's Approach from your graveyard?",
    "If you do, search your library for a Sphinx creature card, put it onto the battlefield, then shuffle",
    "Exile them",
    "Don't exile",
)
.no_default();

/// A Sphinx card (CR 205.3m): the creature type is read through the spell's words, so a text-changing effect on the spell
/// changes the type searched for (CR 612). The words are the spell's last-known ones once it has been exiled (CR 400.7).
fn sphinx(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let kind = q.words(choice.source).subtype(Subtype::Sphinx);
    Verdict::each(choice, |card| q.chars(card).has_subtype(kind))
}

/// "Search your library for a Sphinx creature card, put it onto the battlefield, then shuffle" (CR 701.23).
fn search(g: &mut dyn Game, you: PlayerId, source: ObjectId) -> Result<(), Suspend> {
    g.end_sentence();
    let rule = Selection::new(up_to(1)).with(select::CREATURE).with(part(&(sphinx as SelectFn), 0, "a Sphinx")).called("Sphinx creature");
    find_to_battlefield(g, you, source, rule, false)?;
    Ok(())
}

/// "Draw two cards. Then you may exile this spell and four cards named Sphinx's Approach from your graveyard. If you do,
/// search your library for a Sphinx creature card, put it onto the battlefield, then shuffle."
///
/// The draw is two cards (CR 121.2). The exile is one action of five objects - this spell and exactly four cards whose
/// name is Sphinx's Approach (CR 201.2) - so it is offered only when the graveyard holds all four (CR 608.2d), its controller
/// choosing which four when there are more, and it is one exile (CR 701.13, 101.4): the spell leaves the stack for exile
/// rather than its owner's graveyard and the four cards leave the graveyard together. "If you do" (CR 608.2c) searches only
/// when all five objects were exiled; the search is for a Sphinx creature card, put onto the battlefield, then shuffles
/// (CR 701.23); a search that finds nothing still shuffles.
///
/// "A deck can have any number of cards named Sphinx's Approach" is a deck-construction rule, not an ability: it is not
/// in any game and no line of code here.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
    g.end_sentence();
    let named: Vec<ObjectId> = g
        .state()
        .zone(Zone::graveyard(you))
        .iter()
        .copied()
        // The literal name the card's text prints, not whatever name this particular object shows: a copy keeps
        // Sphinx's Approach's text (CR 707.2) whatever its own name is.
        .filter(|&card| object_names(g, card).contains(&oracle::SPHINXS_APPROACH.name))
        .collect();
    if named.len() < 4 {
        return Ok(());
    }
    if !effects::ask_may(g, you, item.object, Purpose::Ask(&EXILE_AND_SEARCH))? {
        return Ok(());
    }
    // Exactly four of the named cards, the controller's choice (CR 608.2d).
    let rule = select::cards(select::exactly(4)).with(select::named(g.card_id(&CARD))).called("cards");
    let mut exiled = choose::select(g, you, item.object, Purpose::Exile, named, rule)?;
    exiled.push(item.object);
    // "If you do": only when every one of them was exiled (CR 608.2c).
    if g.exile_all(&exiled).len() == exiled.len() { search(g, you, item.object)?; }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::SPHINXS_APPROACH };
