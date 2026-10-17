use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        // "Spend this mana only to cast spells with mana value 5 or greater or spells with {X} in their mana costs"
        // (CR 106.6, 202.3, 107.3): the two mana may pay different spells and any part of the cost of a spell with {X}
        // (its rulings).
        mana_ability(TAP, add_each(&[Mana::G, Mana::U]).only_for(ManaRestriction::mana_value_at_least_or_x(5)), "{T}: Add {G}{U}. Spend this mana only to cast spells with mana value 5 or greater or spells with {X} in their mana costs."),
        ability(cost("{U}").and(TAP), effects::loot, "{U}, {T}: Draw a card, then discard a card."),
    ],
    ..oracle::TROYAN_GUTSY_EXPLORER
};
