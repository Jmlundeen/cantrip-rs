use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(cost("{1}, {T}").and(SACRIFICE_THIS), add_any_color(1), "{1}, {T}, Sacrifice this artifact: Add one mana of any color."),
        // Sacrificed for its own mana, it still draws (CR 603.10a).
        triggered(THIS_DIES, |g, item| cantrip_mechanics::effects::draw(g, item, 1), "When Chromatic Star is put into a graveyard from the battlefield, draw a card."),
    ],
    ..oracle::CHROMATIC_STAR
};
