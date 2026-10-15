//! Abilities the rules give an object because of what it is rather than what it prints: a basic land type's mana
//! ability (CR 305.6). Held as a card of their own in every `CardTable`, so an `AbilityRef` names them as it names
//! any other ability.

use cantrip_state::*;

use crate::card::*;
use crate::game::{Game, Query, Suspend};

fn add(g: &mut dyn Game, asker: &Asker, mana: Mana) -> Result<(), Suspend> {
    g.add_mana(asker.controller, mana, 1);
    Ok(())
}

fn add_w(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::W)
}
fn add_u(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::U)
}
fn add_b(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::B)
}
fn add_r(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::R)
}
fn add_g(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::G)
}

const fn tap_for(effect: ManaEffect, mana: Mana, text: &'static str) -> AbilityDef {
    let mut adds = [0u8; 6];
    adds[mana as usize] = 1;
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(effect),
        adds: ManaPool(adds),
        ..ActivatedDef::BLANK
    })
}

fn draw_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

fn crown_subject(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(p) = item.subject {
        g.become_monarch(p);
    }
    Ok(())
}

fn venture(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.venture(item.controller)
}

fn initiative_subject(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(p) = item.subject {
        g.take_initiative(p);
    }
    Ok(())
}

const fn inherent(text: &'static str, effect: Effect) -> AbilityDef {
    AbilityDef::Triggered(TriggeredDef { text, effect, ..TriggeredDef::BLANK })
}

/// The rules' own triggered abilities, which no object has and which the rules themselves put on the stack
/// (CR 725.2, 726.2). The first five abilities are the basic land types' mana abilities.
pub const MONARCH_DRAW: u8 = 5;
pub const MONARCH_TAKEN: u8 = 6;
pub const INITIATIVE_VENTURE: u8 = 7;
pub const INITIATIVE_TAKEN: u8 = 8;
/// A Siege's defeat ability (CR 310.12b).
pub const SIEGE_DEFEAT: u8 = 9;
/// Suspend's two triggered abilities (CR 702.62a): the upkeep one and the one for the last counter removed.
pub const SUSPEND_UPKEEP: u8 = 10;
pub const SUSPEND_CAST: u8 = 11;

/// "At the beginning of your upkeep, if this card is suspended, remove a time counter from it." (CR 702.62a)
fn suspend_upkeep(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    if !crate::game::is_live(g, card) || !g.obj(card).zone.is(ZoneKind::Exile) || g.obj(card).counters(CounterKind::Time) <= 0 {
        return Ok(());
    }
    g.add_counters(card, CounterKind::Time, -1);
    // "When the last is removed" is a triggered ability of its own, put on the stack the next time a player gets priority.
    if g.obj(card).counters(CounterKind::Time) == 0 {
        let ability = AbilityRef { card: g.card_id(&INTRINSIC), index: SUSPEND_CAST };
        g.reflexive_trigger(card, item.controller, ability, Target::None);
    }
    Ok(())
}

/// "When the last is removed and the card is exiled, you may cast it without paying its mana cost. If you don't, it
/// remains exiled." (CR 702.62a)
fn suspend_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    if !crate::game::is_live(g, card) || !g.obj(card).zone.is(ZoneKind::Exile) || !g.could_cast_during_resolution(card) {
        return Ok(());
    }
    if g.ask(item.controller, RequestKind::May { source: card, purpose: Purpose::Cast })? == Answer::Yes {
        g.cast_during_resolution(card, Some(ManaCost::ZERO))?;
    }
    Ok(())
}

/// "When the last defense counter is removed from this permanent, exile it, then you may cast it transformed without
/// paying its mana cost" (CR 310.12b). "It" is the permanent the ability came from; if that has left the battlefield
/// since, it is a new object and nothing is exiled. Declined, or impossible to cast, the card stays in exile.
fn siege_defeated(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !crate::game::is_live(g, item.object) || !g.is_on_battlefield(item.object) {
        return Ok(());
    }
    let card = g.exile(item.object);
    if !g.obj(card).zone.is(ZoneKind::Exile) || !g.could_cast_transformed_during_resolution(card) {
        return Ok(());
    }
    if g.ask(item.controller, RequestKind::May { source: card, purpose: Purpose::Cast })? == Answer::Yes {
        g.cast_transformed_during_resolution(card)?;
    }
    Ok(())
}

/// Index `Mana::W as u8` and on: "{T}: Add {W}." for Plains, and so on (CR 305.6).
pub static INTRINSIC: CardDef = CardDef {
    name: "(rules)",
    has_cost: false,
    abilities: &[
        tap_for(add_w, Mana::W, "{T}: Add {W}."),
        tap_for(add_u, Mana::U, "{T}: Add {U}."),
        tap_for(add_b, Mana::B, "{T}: Add {B}."),
        tap_for(add_r, Mana::R, "{T}: Add {R}."),
        tap_for(add_g, Mana::G, "{T}: Add {G}."),
        inherent("At the beginning of the monarch's end step, that player draws a card.", draw_one),
        inherent("Whenever a creature deals combat damage to the monarch, its controller becomes the monarch.", crown_subject),
        inherent("Whenever a player takes the initiative, that player ventures into Undercity.", venture),
        inherent(
            "Whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the controller of those creatures takes the initiative.",
            initiative_subject,
        ),
        inherent("When the last defense counter is removed from this permanent, exile it. Then you may cast it transformed without paying its mana cost.", siege_defeated),
        inherent("Suspend: at the beginning of your upkeep, if this card is suspended, remove a time counter from it.", suspend_upkeep),
        inherent("Suspend: when the last time counter is removed from this card, if it's exiled, you may cast it without paying its mana cost.", suspend_cast),
    ],
    ..CardDef::BLANK
};

/// "Turn it face up" (CR 701.40b, 708.7): a face-down permanent that is a creature card may be turned face up any time
/// its controller has priority, by paying its mana cost. A special action (CR 116.2): no stack. The cost is the card's
/// own, which the rules read off the permanent (`actions::special_action`), not off this ability.
fn turn_face_up(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.turn_face_up(item.object);
    Ok(())
}

/// Whether the face-down permanent is a creature card with a mana cost that can be paid to turn it face up.
fn can_turn_face_up(q: &dyn Query, asker: &Asker) -> bool {
    let obj = q.obj(asker.source);
    let def = q.card(obj.card);
    obj.face_down && def.types.has(CardType::Creature) && def.has_cost
}

/// What a face-down permanent is (CR 708.2a): a 2/2 creature with no name, no mana cost, no color, no subtypes and no
/// abilities but the one to turn it face up. A face-down permanent has this as its copiable values (CR 707.2).
pub static FACE_DOWN: CardDef = CardDef {
    name: "",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Turn this permanent face up by paying its mana cost.",
        effect: turn_face_up,
        special_action: true,
        condition: Some(can_turn_face_up),
        ..ActivatedDef::BLANK
    })],
    ..CardDef::BLANK
};
