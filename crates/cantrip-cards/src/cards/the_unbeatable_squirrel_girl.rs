use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Create a 1/1 green Squirrel creature token."
fn make_a_squirrel(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::SQUIRREL, 1)?;
    Ok(())
}

/// "Create X 1/1 green Squirrel creature tokens, where X is the number of Squirrels you control": X is counted as the ability
/// resolves (CR 608.2h), before any token exists, and the tokens are made together (CR 101.4). "Squirrel" is a word of this card's text (CR 612).
fn make_squirrels(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let squirrel = g.words(item.object).subtype(Subtype::Squirrel);
    let x = g.permanents_of(you).into_iter().filter(|&id| g.chars(id).has_subtype(squirrel)).count() as u32;
    tokens::create(g, you, &tokens::SQUIRREL, x)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS_OR_ATTACKS, make_a_squirrel, "Whenever The Unbeatable Squirrel Girl enters or attacks, create a 1/1 green Squirrel creature token."),
        ability(
            cost("{1}{G}{G}{G}"),
            make_squirrels,
            "{1}{G}{G}{G}: Create X 1/1 green Squirrel creature tokens, where X is the number of Squirrels you control.",
        ),
    ],
    ..oracle::THE_UNBEATABLE_SQUIRREL_GIRL
};

pub static EXTRA: &[&CardDef] = &[&tokens::SQUIRREL];
