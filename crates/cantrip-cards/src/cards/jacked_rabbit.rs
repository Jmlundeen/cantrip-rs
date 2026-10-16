use cantrip_api::*;
use cantrip_mechanics::{ravenous, tokens::{self, RABBIT}};
use cantrip_oracle as oracle;

/// Create a number of 1/1 white Rabbit creature tokens equal to this creature's power.
fn make_rabbits(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u32;
    if power > 0 {
        tokens::create(g, item.controller, &RABBIT, power)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(ravenous::enters),
    abilities: &[
        ravenous::draw_trigger("When Jacked Rabbit enters, if X is 5 or more, draw a card."),
        triggered(cantrip_mechanics::triggers::THIS_ATTACKS, make_rabbits, "Whenever Jacked Rabbit attacks, create a number of 1/1 white Rabbit creature tokens equal to its power."),
    ],
    ..oracle::JACKED_RABBIT
};

pub static EXTRA: &[&CardDef] = &[&RABBIT];
