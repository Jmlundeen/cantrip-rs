use cantrip_api::*;
use cantrip_mechanics::{choose, same_name};
use cantrip_oracle as oracle;

/// "As an additional cost to cast this spell, you may collect evidence 6" (CR 701.59a, 601.2b): announced like any
/// optional additional cost; the spell is `kicked` when it was paid.
const OPTIONAL_COST: OptionalCost = OptionalCost { cost: Cost { collect_evidence: 6, ..Cost::NONE } };

const LABELS: &[&str] = &["Cast without collecting evidence", "Collect evidence 6"];

/// The library part of the search: you search the owner's library for any number of cards with the exiled card's name
/// (CR 701.23). The searcher is the one asked; the library is the owner's. Nothing is asked of an empty library, and the
/// search is not a search of the searcher's own library, so no `searched` event is made for it.
fn search_owners_library(g: &mut dyn Game, you: PlayerId, owner: PlayerId, source: ObjectId, exiled: ObjectId) -> Result<Vec<ObjectId>, Suspend> {
    let mut library: Vec<ObjectId> = g.state().zone(Zone::library(owner)).to_vec();
    library.reverse();
    let eligible: Vec<ObjectId> = library.iter().copied().filter(|&c| same_name(g, exiled, c)).collect();
    if library.is_empty() {
        return Ok(Vec::new());
    }
    let answer = g.ask(you, RequestKind::SearchLibrary { source, library, eligible, count: u8::MAX, optional: true })?;
    Ok(if let Answer::Cards(cards) = answer { cards } else { Vec::new() })
}

/// "Destroy all creatures. If evidence was collected, exile a card from an opponent's graveyard. Then search its
/// owner's graveyard, hand, and library for any number of cards with that name and exile them. That player shuffles,
/// then draws a card for each card exiled from their hand this way."
///
/// The creatures go together (they may be regenerated), and the ones of the opponents land in their graveyards first,
/// so they may be the card exiled. You choose the card from among all your opponents' graveyards, and you do the
/// searching, in the order the text names the zones; each zone is a search for any number (CR 701.23), so cards of the
/// name may be left. The card exiled is still the object whose name is asked after it has moved (CR 400.7), and the
/// found cards are exiled together. With no card to exile, "its owner" is nobody and nothing more happens.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let creatures: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    g.destroy_all(&creatures);
    if item.kicked == 0 {
        return Ok(());
    }
    let opponents: Vec<PlayerId> = g.state().opponents(you).collect();
    let graveyards: Vec<ObjectId> = opponents.iter().flat_map(|&p| g.state().zone(Zone::graveyard(p)).to_vec()).collect();
    let Some(chosen) = choose::card(g, you, item.object, Purpose::Exile, graveyards)? else { return Ok(()) };
    let owner = g.obj(chosen).owner;
    let exiled = g.exile(chosen);
    let mut found = Vec::new();
    let graveyard: Vec<ObjectId> = g.state().zone(Zone::graveyard(owner)).iter().copied().filter(|&c| same_name(g, exiled, c)).collect();
    found.extend(choose::cards(g, you, item.object, Purpose::Exile, graveyard, u8::MAX, true)?);
    let hand: Vec<ObjectId> = g.state().zone(Zone::hand(owner)).iter().copied().filter(|&c| same_name(g, exiled, c)).collect();
    let from_hand = choose::cards(g, you, item.object, Purpose::Exile, hand, u8::MAX, true)?;
    found.extend(from_hand.iter().copied());
    found.extend(search_owners_library(g, you, owner, item.object, exiled)?);
    g.exile_all(&found);
    g.shuffle(owner);
    g.draw(owner, from_hand.len() as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    optional_cost: Some(OPTIONAL_COST),
    labels: LABELS,
    ..oracle::DEADLY_COVER_UP
};
