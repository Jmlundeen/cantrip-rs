use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

fn add_any(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    Ok(())
}

/// "Sacrifice a creature: Add one mana of any color." A mana ability whose sacrifice comes with the activation.
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice a creature: Add one mana of any color.",
        cost: Cost::sacrifice(1, filters::creature),
        mana: Some(add_any),
        any_color: true,
        adds_any: 1,
        ..ActivatedDef::BLANK
    })],
    ..oracle::PHYREXIAN_ALTAR
};
