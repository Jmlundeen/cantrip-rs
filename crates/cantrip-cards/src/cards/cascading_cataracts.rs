use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{5}, {T}: Add five mana in any combination of colors" (CR 106.1b, 605.3b, 608.2d): each of the five is a color of its own
/// choosing, chosen as the ability resolves, the same color five times included.
fn add_five_colors(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    for color in cantrip_mechanics::choose_colors(g, asker.controller, asker.source, ColorSet(0b11111), 5)? {
        g.add_mana(asker.controller, color.mana(), 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(cost("{5}, {T}"), add_by(add_five_colors), "{5}, {T}: Add five mana in any combination of colors."),
    ],
    ..oracle::CASCADING_CATARACTS
};
