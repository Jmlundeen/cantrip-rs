use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "Add {U}{U}, {U}{B}, or {B}{B}" (CR 106.1b, 605.3): two mana, each a color of its own choosing from the two, chosen as
/// the mana is added.
fn add_two(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let choices = ColorSet(ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Black).0);
    if let Answer::Colors(colors) = g.ask(asker.controller, RequestKind::ChooseColors { source: asker.source, choices, count: 2 })? {
        for color in colors {
            g.add_mana(asker.controller, color.mana(), 1);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{U/B}, {T}: Add {U}{U}, {U}{B}, or {B}{B}.",
            cost: Cost {
                mana: ManaCost { hybrid: [Some((Mana::U, Mana::B)), None, None, None, None, None], ..mana(0, 0, 0, 0, 0, 0) },
                tap: true,
                ..Cost::NONE
            },
            mana: Some(add_two),
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::SUNKEN_RUINS
};
