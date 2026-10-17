use cantrip_api::*;
use cantrip_mechanics::{effects, statics};
use cantrip_oracle as oracle;

/// "Activate only if you have seven or more cards in your hand" (CR 602.5b): checked as the ability is activated.
fn seven_cards_in_hand(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::hand(asker.controller)).len() >= 7
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, |g, item| effects::draw(g, item, 1), "{T}: Draw a card. Activate only if you have seven or more cards in your hand.")
        .condition(seven_cards_in_hand)],
    // "{T}: Add two mana of any one color. Spend this mana only to cast instant and sorcery spells.", the mana ability the lands are given.
    delayed: &[mana_ability(
        TAP,
        add_any_color(2).only_for(ManaRestriction::INSTANT_OR_SORCERY_SPELLS),
        "{T}: Add two mana of any one color. Spend this mana only to cast instant and sorcery spells.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_lands,
        statics::grant_delayed_ability,
        "Lands you control have \"{T}: Add two mana of any one color. Spend this mana only to cast instant and sorcery spells.\"",
    )],
    ..oracle::RESONATING_LUTE
};
