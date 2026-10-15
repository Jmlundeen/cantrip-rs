use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

fn add_one_of_any_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::sunburst),
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Remove a charge counter from this artifact: Add one mana of any color.",
        cost: Cost { remove_counters: Some((CounterKind::Charge, 1)), ..Cost::NONE },
        mana: Some(add_one_of_any_color),
        any_color: true,
        adds_any: 1,
        ..ActivatedDef::BLANK
    })],
    ..oracle::PENTAD_PRISM
};
