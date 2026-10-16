use cantrip_api::*;
use cantrip_oracle as oracle;

/// The colors among the legendary creatures and planeswalkers its controller controls (CR 105.2), read through the
/// layers each time the ability is offered. With none there is no color to choose: the ability is still offered and adds
/// no mana (CR 106.5).
fn colors_among_legends(q: &dyn Query, asker: &Asker) -> ColorSet {
    let mut colors = ColorSet::NONE;
    for id in q.permanents_of(asker.controller) {
        let c = q.chars(id);
        if c.supertypes.legendary && (c.is(CardType::Creature) || c.is(CardType::Planeswalker)) {
            colors = ColorSet(colors.0 | c.colors.0);
        }
    }
    colors
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color among legendary creatures and planeswalkers you control.")
        .colors_from(colors_among_legends)],
    ..oracle::MOX_AMBER
};
