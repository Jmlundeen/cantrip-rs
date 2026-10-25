use cantrip_api::*;
use cantrip_mechanics::{empower, statics, tokens, tokens::BEAST_4_4_TRAMPLE, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When Way of the Wildspeaker enters, empower Jace 7."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 7)
}

/// "Planeswalkers you control have '-4: Create a 4/4 green Beast creature token with trample.'"
fn create_beast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BEAST_4_4_TRAMPLE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, empower_jace, "When Way of the Wildspeaker enters, empower Jace 7.")],
    delayed: &[ability(loyalty(-4), create_beast, "\u{2212}4: Create a 4/4 green Beast creature token with trample.")],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_planeswalkers,
        statics::grant_delayed_ability,
        "Planeswalkers you control have \"\u{2212}4: Create a 4/4 green Beast creature token with trample.\"",
    )],
    ..oracle::WAY_OF_THE_WILDSPEAKER
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE, &BEAST_4_4_TRAMPLE];
