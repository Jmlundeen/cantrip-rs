use cantrip_api::*;
use cantrip_mechanics::{choose, select};
use cantrip_mechanics::library::{Find, object_names, search_to_battlefield};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

const EXILE_AND_SEARCH: Question = Question::new(
    "Exile",
    "exile this spell and four cards named Sphinx's Approach from your graveyard?",
    "If you do, search your library for a Sphinx creature card, put it onto the battlefield, then shuffle",
    "Exile them",
    "Don't exile",
)
.no_default();

/// "Draw two cards. Then you may exile this spell and four cards named Sphinx's Approach from your graveyard. If you do,
/// search your library for a Sphinx creature card, put it onto the battlefield, then shuffle."
///
/// The draw is two cards (CR 121.2). The exile is one action of five objects — this spell and exactly four cards whose
/// name is Sphinx's Approach (CR 201.2) — so it is offered only when the graveyard holds all four (CR 608.2d), and it is
/// one exile (CR 701.13, 101.4): the spell leaves the stack for exile rather than its owner's graveyard and the four cards
/// leave the graveyard together. "If you do" then searches for a Sphinx creature card, puts it onto the battlefield and
/// shuffles (CR 701.23); a search that finds nothing still shuffles. A spell that is a copy has no card to exile but does
/// the rest.
///
/// "A deck can have any number of cards named Sphinx's Approach" is a deck-construction rule, not an ability: it is not
/// in any game and no line of code here.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
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
    // The spell's resolving action exiles it and exactly four named cards; when more than four are available, its controller
    // chooses which four (CR 608.2c).
    let chosen = if named.len() == 4 {
        named
    } else {
        let rule = Selection::new(select::exactly(4))
            .with(select::named(g.card_id(&CARD)))
            .called("cards");
        choose::select(g, you, item.object, Purpose::Exile, named, rule)?
    };
    let mut exiled = chosen;
    exiled.push(item.object);
    let moved = g.exile_all(&exiled);
    // The spell is the last object moved; the search's source is the card it became in exile (CR 400.7).
    let source = moved.last().copied().unwrap_or(item.object);
    search_to_battlefield(g, you, source, Find::ONE, false, |q, card| {
        let c = q.chars(card);
        c.is(CardType::Creature) && c.subtypes.has(Subtype::Sphinx)
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::SPHINXS_APPROACH };
