use cantrip_api::*;
use cantrip_mechanics::statics::{other_of_subtype_you_control, plus_pt};
use cantrip_mechanics::triggers::YOU_DRAW_YOUR_SECOND_CARD;
use cantrip_mechanics::{lands, tokens};
use cantrip_oracle as oracle;

/// "Seek out — Whenever you draw your second card each turn, create a 1/1 red Officer creature token." (CR 603.2): the
/// shared OFFICER token definition, created once under its controller, exactly as Academy Instructor makes.
fn officer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OFFICER, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        other_of_subtype_you_control::<{ lands::sub(Subtype::Officer) }>,
        plus_pt::<1, 1>,
        "Other Officers you control get +1/+1.",
    )],
    abilities: &[triggered(
        YOU_DRAW_YOUR_SECOND_CARD,
        officer,
        "Seek out — Whenever you draw your second card each turn, create a 1/1 red Officer creature token.",
    )],
    ..oracle::WORF_SON_OF_MOGH
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
