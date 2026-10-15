use cantrip_api::*;
use cantrip_mechanics::{bargain, choose, search_library, effects};
use cantrip_oracle as oracle;

/// "Search your library for a card, exile it face down, then shuffle. If this spell was bargained, you may cast the
/// exiled card without paying its mana cost if that spell's mana value is 4 or less. Put the exiled card into your hand
/// if it wasn't cast this way."
///
/// The search names no quality, so a card is found whenever the library has one (CR 701.23b). The searcher looked at it
/// before exiling it face down, and may go on looking (CR 406.3). It is cast face up (CR 406.3a-b) with the mana value
/// asked of the resulting spell, X being zero (CR 118.9, 202.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let found = search_library(g, you, you, item.object, 1, false, |_, _| true)?;
    let mut exiled = None;
    if let Some(&card) = found.first() {
        let new = g.exile_face_down(&[card], None);
        g.look_at(you, &new);
        exiled = new.first().copied().filter(|&c| g.obj(c).zone.is(ZoneKind::Exile));
    }
    g.shuffle(you);
    let Some(card) = exiled else { return Ok(()) };
    if bargain::bargained(item) {
        // "If that spell's mana value is 4 or less": the face it is cast as is the spell (CR 709.3b, 715.3b).
        let ways = g.cast_ways_during_resolution(card).into_iter().filter(|w| w.mana_value <= 4).collect::<Vec<_>>();
        if !ways.is_empty()
            && effects::ask_may(g, you, card, Purpose::Cast)?
            && let Some(way) = choose::cast_way(g, you, card, ways)?
            && g.cast_during_resolution_as(card, CastTerms { instead: Some(ManaCost::ZERO), way: Some(way.way), ..CastTerms::default() })?
        {
            return Ok(());
        }
    }
    if is_live(g, card) && g.obj(card).zone.is(ZoneKind::Exile) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    optional_cost: Some(bargain::OPTIONAL_COST),
    labels: bargain::LABELS,
    ..oracle::BESEECH_THE_MIRROR
};
