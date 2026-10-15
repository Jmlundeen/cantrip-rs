use cantrip_api::*;
use cantrip_oracle as oracle;

fn three_of_one_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Discard your hand, Sacrifice this artifact: Add three mana of any one color. Activate only as an instant.",
        cost: Cost { discard_hand: true, sacrifice_self: true, ..Cost::NONE },
        mana: Some(three_of_one_color),
        any_color: true,
        adds_any: 3,
        ..ActivatedDef::BLANK
    })],
    ..oracle::LIONS_EYE_DIAMOND
};
