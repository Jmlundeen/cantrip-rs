use cantrip_api::*;
use cantrip_oracle as oracle;

fn one_of_any_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}, Sacrifice Lotus Petal: Add one mana of any color.",
        cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
        mana: Some(one_of_any_color),
        any_color: true,
        adds_any: 1,
        ..ActivatedDef::BLANK
    })],
    ..oracle::LOTUS_PETAL
};
