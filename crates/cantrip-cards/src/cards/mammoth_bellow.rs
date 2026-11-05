use cantrip_api::*;
use cantrip_mechanics::{harmonize, tokens};
use cantrip_oracle as oracle;

/// "Create a 5/5 green Elephant creature token" (CR 111.1).
fn create_an_elephant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ELEPHANT_5_5, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], create_an_elephant),
    ways: &[harmonize::way(mana("{5}{G}{U}{R}"), "Harmonize {5}{G}{U}{R}")],
    ..oracle::MAMMOTH_BELLOW
};

pub static EXTRA: &[&CardDef] = &[&tokens::ELEPHANT_5_5];
