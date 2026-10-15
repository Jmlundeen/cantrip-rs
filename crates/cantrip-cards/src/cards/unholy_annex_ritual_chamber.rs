//! Unholy Annex // Ritual Chamber, a Room (CR 709.5): a permanent with two doors, each a half of the card. Cast one half
//! and only that door starts unlocked; the other is unlocked by paying its mana cost as a special action (CR 709.5e).
//! The engine gives the Room its name, mana cost and abilities by its unlocked doors (`layers::doors`); each door here is
//! only what its text does. Other Rooms this makes possible are every one in Duskmourn: Bottomless Pool // Locker Room,
//! Crude Abattoir // Unsavory Kitchen, Funeral Room // Awakening Hall, Derelict Attic // Widow's Walk and the rest.

use cantrip_api::*;
use cantrip_mechanics::triggers::{self, your_end_step};
use cantrip_oracle as oracle;

/// The 6/6 black Demon creature token with flying.
pub static DEMON: CardDef = CardDef {
    name: "Demon",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Black)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Demon]),
    power: Some(6),
    toughness: Some(6),
    keywords: Keywords::of(&[Keyword::Flying]),
    text: "Flying",
    ..CardDef::BLANK
};

/// Unholy Annex: "At the beginning of your end step, draw a card. If you control a Demon, each opponent loses 2 life and
/// you gain 2 life. Otherwise, you lose 2 life." Whether you control a Demon is asked as the ability resolves, after the
/// draw, by the Demon type as this Room's text has it (CR 612).
fn annex(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 1);
    let demon = g.words(item.object).subtype(Subtype::Demon);
    if g.permanents_of(you).into_iter().any(|id| g.chars(id).has_subtype(demon)) {
        g.each_opponent_loses_life(you, 2);
        g.gain_life(you, 2);
    } else {
        g.lose_life(you, 2);
    }
    Ok(())
}

/// Ritual Chamber: "When you unlock this door, create a 6/6 black Demon creature token with flying."
fn chamber(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &DEMON, 1)?;
    Ok(())
}

pub static UNHOLY_ANNEX: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "At the beginning of your end step, draw a card. If you control a Demon, each opponent loses 2 life and you gain 2 life. Otherwise, you lose 2 life.",
        listens: EventMask::STEP_BEGAN,
        matches: your_end_step,
        effect: annex,
        ..TriggeredDef::BLANK
    })],
    ..oracle::UNHOLY_ANNEX
};

pub static RITUAL_CHAMBER: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When you unlock this door, create a 6/6 black Demon creature token with flying.",
        listens: EventMask::DOOR_UNLOCKED,
        matches: triggers::you_unlock_this_right_door,
        effect: chamber,
        ..TriggeredDef::BLANK
    })],
    ..oracle::RITUAL_CHAMBER
};

pub static CARD: CardDef = CardDef { split: Some((&UNHOLY_ANNEX, &RITUAL_CHAMBER)), ..oracle::UNHOLY_ANNEX_RITUAL_CHAMBER };

pub static EXTRA: &[&CardDef] = &[&UNHOLY_ANNEX, &RITUAL_CHAMBER, &DEMON];
