use cantrip_api::*;
use cantrip_mechanics::tokens::MONK;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{PROWESS, noncreature_spell_you_cast};
use cantrip_oracle as oracle;

fn make_monk(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &MONK, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        PROWESS,
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you cast a noncreature spell, create a 1/1 white Monk creature token with prowess.",
            listens: EventMask::SPELL_CAST,
            matches: noncreature_spell_you_cast,
            effect: make_monk,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::MONASTERY_MENTOR
};

pub static EXTRA: &[&CardDef] = &[&MONK];
