use cantrip_api::*;
use cantrip_mechanics::{empower, statics, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When Way of the Pyromancer enters, empower Jace 2."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 2)
}

/// "[+1]: Add {R}.", the loyalty ability the planeswalkers are given. It is not a mana ability (CR 605.1a, 606.1): it uses the
/// stack, and like any loyalty ability it is activated once per planeswalker each turn, at sorcery speed (CR 606.3). The
/// planeswalker that has it is the object that activates it (the item's source), and the mana goes to its activator.
fn add_red(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::R, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, empower_jace, "When Way of the Pyromancer enters, empower Jace 2.")],
    delayed: &[ability(loyalty(1), add_red, "+1: Add {R}.")],
    statics: &[static_ability(Layer::Ability, statics::your_planeswalkers, statics::grant_delayed_ability, "Planeswalkers you control have \"[+1]: Add {R}.\"")],
    ..oracle::WAY_OF_THE_PYROMANCER
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
