use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::{scry, top_of_library, triggers};
use cantrip_oracle as oracle;

/// The triggered ability, second in the card's list: what "do this only once each turn" is recorded against.
fn look_ability(q: &dyn Query, source: ObjectId) -> AbilityRef {
    AbilityRef { card: q.chars(source).card, index: 1 }
}

/// "{1}, {T}: Scry 2." (CR 701.22)
fn scry_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 2)
}

/// "Whenever you scry or surveil" (CR 701.22d, 701.25d: once the scry or surveil is complete, whatever made its
/// controller do it). "Do this only once each turn" (CR 603.2h): "This ability triggers only if its source's controller
/// has not yet taken the indicated action that turn" - the controller, so the action is looked for among what that
/// player did this turn, with this ability of any Planetarium, a Planetarium that left and came back included (the
/// ruling: "Once you choose to cast the top card of your library, Planetarium of Wan Shi Tong's ability won't trigger
/// again that turn").
fn you_scry_or_surveil_not_yet_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let mine = look_ability(q, me.source);
    let you = me.controller;
    let cast = q.history().this_turn().iter().any(|r| matches!(r.event, Event::Done { ability, player, .. } if ability == mine && player == you));
    if cast {
        return None;
    }
    triggers::you_scry_or_surveil(q, me, event)
}

/// "Look at the top card of your library. You may cast that card without paying its mana cost. Do this only once each
/// turn." The card is looked at whatever happens next (an empty library shows nothing). It is cast as the ability
/// resolves or not at all (CR 608.2g, the ruling), its targets chosen and its mandatory additional costs paid as for any
/// cast, X being 0 (CR 118.9, 107.3b); a land can't be cast. "Only once each turn" limits the triggering alone (CR
/// 603.2h): a trigger already on the stack when a card is cast with another still offers its cast. Its source having
/// left the battlefield changes nothing (CR 113.7a).
fn look_and_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let Some(&top) = top_of_library(g, you, 1).first() else { return Ok(()) };
    g.look_at(you, &[top]);
    if !g.could_cast_during_resolution(top) {
        return Ok(());
    }
    if ask_may(g, you, top, Purpose::CastFree)? && g.cast_during_resolution(top, Some(ManaCost::ZERO))? {
        g.note_done(item.object, item.ability);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}, {T}"), scry_two, "{1}, {T}: Scry 2."),
        triggered(when_asked(EventMask(EventMask::SCRIED.0 | EventMask::SURVEILLED.0), you_scry_or_surveil_not_yet_cast), look_and_cast, "Whenever you scry or surveil, look at the top card of your library. You may cast that card without paying its mana cost. Do this only once each turn. (Look at the card after you scry or surveil.)"),
    ],
    ..oracle::PLANETARIUM_OF_WAN_SHI_TONG
};
