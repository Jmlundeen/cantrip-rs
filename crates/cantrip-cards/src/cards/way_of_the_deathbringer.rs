use cantrip_api::*;
use cantrip_mechanics::{effects, empower, select, statics, tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When Way of the Deathbringer enters, empower Jace 5."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 5)
}

/// "[-2]: You may sacrifice a creature. If you do, create a 4/4 green Beast creature token with trample." The ability the
/// planeswalkers are given: its activator chooses a creature they control as it resolves (not a cost, CR 608.2d) or none; a
/// creature that can't be sacrificed is not one to choose (CR 101.2), and the token is made only if one was sacrificed (CR 608.2c).
fn sacrifice_for_beast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if !effects::may_sacrifice(g, you, item.object, select::creature_cards(select::up_to(1)))?.is_empty() {
        g.end_sentence();
        tokens::create(g, you, &tokens::BEAST_4_4_TRAMPLE, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, empower_jace, "When Way of the Deathbringer enters, empower Jace 5.")],
    delayed: &[ability(
        loyalty(-2),
        sacrifice_for_beast,
        "\u{2212}2: You may sacrifice a creature. If you do, create a 4/4 green Beast creature token with trample.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_planeswalkers,
        statics::grant_delayed_ability,
        "Planeswalkers you control have \"\u{2212}2: You may sacrifice a creature. If you do, create a 4/4 green Beast creature token with trample.\"",
    )],
    ..oracle::WAY_OF_THE_DEATHBRINGER
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE, &tokens::BEAST_4_4_TRAMPLE];
