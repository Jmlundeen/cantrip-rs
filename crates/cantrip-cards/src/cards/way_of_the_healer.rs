use cantrip_api::*;
use cantrip_mechanics::{effects, empower, statics, tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When Way of the Healer enters, empower Jace 5."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 5)
}

/// "[-2]: Create a 2/2 colorless Wizard Soldier creature token named Cadet. Surveil 1." The loyalty ability the planeswalkers are
/// given: the planeswalker that has it is the item's source and its activator creates the token, then surveils (CR 701.25).
fn cadet_then_surveil(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::CADET, 1)?;
    effects::surveil(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, empower_jace, "When Way of the Healer enters, empower Jace 5.")],
    delayed: &[ability(loyalty(-2), cadet_then_surveil, "\u{2212}2: Create a 2/2 colorless Wizard Soldier creature token named Cadet. Surveil 1.")],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_planeswalkers,
        statics::grant_delayed_ability,
        "Planeswalkers you control have \"\u{2212}2: Create a 2/2 colorless Wizard Soldier creature token named Cadet. Surveil 1.\"",
    )],
    ..oracle::WAY_OF_THE_HEALER
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE, &tokens::CADET];
