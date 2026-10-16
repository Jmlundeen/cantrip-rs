use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Add {U}{U}, {U}{B}, or {B}{B}" (CR 106.1b, 605.3): two mana, each a color of its own choosing from the two, chosen as
/// the mana is added.
fn add_two(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let choices = ColorSet(ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Black).0);
    for color in cantrip_mechanics::choose_colors(g, asker.controller, asker.source, choices, 2)? {
        g.add_mana(asker.controller, color.mana(), 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(cost("{U/B}, {T}"), add_by(add_two), "{U/B}, {T}: Add {U}{U}, {U}{B}, or {B}{B}."),
    ],
    ..oracle::SUNKEN_RUINS
};
