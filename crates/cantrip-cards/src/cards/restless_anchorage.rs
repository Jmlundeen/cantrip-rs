use cantrip_api::*;
use cantrip_mechanics::tokens::create_map;
use cantrip_mechanics::triggers;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

/// "Until end of turn, this land becomes a 2/3 white and blue Bird creature with flying. It's still a land."
/// (CR 613.1d, 613.4b, 613.1e)
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let colors = ColorSet(ColorSet::of(Color::White).0 | ColorSet::of(Color::Blue).0);
    lands::becomes_creature_until_end_of_turn(g, item, (2, 3), colors, Subtype::Bird, Keywords::of(&[Keyword::Flying]));
    Ok(())
}

/// "Whenever this land attacks, create a Map token." (CR 111.10s)
fn map(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_map(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::White, Color::Blue]), "{T}: Add {W} or {U}."),
        ability(cost("{1}{W}{U}"), becomes_creature, "{1}{W}{U}: Until end of turn, this land becomes a 2/3 white and blue Bird creature with flying. It's still a land."),
        triggered(triggers::THIS_ATTACKS, map, "Whenever this land attacks, create a Map token."),
    ],
    ..oracle::RESTLESS_ANCHORAGE
};
