use cantrip_api::*;
use cantrip_mechanics::tokens::MONK;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{PROWESS, NONCREATURE_SPELL_YOU_CAST};
use cantrip_oracle as oracle;

fn make_monk(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &MONK, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        PROWESS,
        triggered(NONCREATURE_SPELL_YOU_CAST, make_monk, "Whenever you cast a noncreature spell, create a 1/1 white Monk creature token with prowess."),
    ],
    ..oracle::MONASTERY_MENTOR
};

pub static EXTRA: &[&CardDef] = &[&MONK];
