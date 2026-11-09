use cantrip_api::*;
use cantrip_oracle as oracle;

/// "any color that a land an opponent controls could produce" (CR 106.7, 800.4a): capabilities through the current
/// layers and replacement effects, irrespective of costs or tapped state; only lands of opponents still in the game.
/// Colorless is deliberately absent from the colors returned by the rules query (CR 106.1b).
fn opponents_land_colors(q: &dyn Query, me: &Asker) -> ColorSet {
    let mut colors = ColorSet::NONE;
    for opponent in q.state().opponents(me.controller) {
        for land in q.permanents_of(opponent) {
            if q.chars(land).is(CardType::Land) {
                colors.0 |= q.producible_types(land).0.0;
            }
        }
    }
    colors
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color that a land an opponent controls could produce.")
        .colors_from(opponents_land_colors)],
    ..oracle::EXOTIC_ORCHARD
};
