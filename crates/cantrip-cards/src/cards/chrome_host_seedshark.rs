use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast a noncreature spell, incubate X, where X is that spell's mana value." The spell's mana value as the
/// ability resolves, or as it last was on the stack if it has left (CR 608.2h), an {X} in its cost counting as the X announced
/// (CR 202.3e); a value past what a counter size holds (RX-5) incubates at that limit.
fn incubate_x_where_x_spells(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    let amount = g.mana_value(spell) as i32;
    tokens::incubate(g, item.controller, amount)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::NONCREATURE_SPELL_YOU_CAST_ABOUT_IT,
            incubate_x_where_x_spells,
            "Whenever you cast a noncreature spell, incubate X, where X is that spell's mana value.",
        ),
    ],
    ..oracle::CHROME_HOST_SEEDSHARK
};

pub static EXTRA: &[&CardDef] = &[&tokens::INCUBATOR, &tokens::PHYREXIAN];
