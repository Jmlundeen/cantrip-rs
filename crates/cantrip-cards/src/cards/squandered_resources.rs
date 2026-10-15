use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Add one mana of any type the sacrificed land could produce (CR 106.7): the core checked the type named against the
/// land, and names colorless as no color at all.
fn add_one(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.map_or(Mana::C, |c| c.mana()), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice a land: Add one mana of any type the sacrificed land could produce.",
        cost: Cost::sacrifice(1, targets::filters::land),
        mana: Some(add_one),
        any_color: true,
        adds_any: 1,
        colors_from_sacrifice: true,
        ..ActivatedDef::BLANK
    })],
    ..oracle::SQUANDERED_RESOURCES
};
