use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let colors = ColorSet(ColorSet::of(Color::White).0 | ColorSet::of(Color::Blue).0);
    lands::becomes_creature_until_end_of_turn(g, item, (4, 4), colors, Subtype::Elemental, Keywords::of(&[Keyword::Flying, Keyword::Vigilance]));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::White, Color::Blue]), "{T}: Add {W} or {U}."),
        ability(cost("{3}{W}{U}"), becomes_creature, "{3}{W}{U}: Until end of turn, this land becomes a 4/4 white and blue Elemental creature with flying and vigilance. It's still a land."),
    ],
    ..oracle::CELESTIAL_COLONNADE
};
