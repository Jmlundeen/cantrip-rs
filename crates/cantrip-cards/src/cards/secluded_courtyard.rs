use cantrip_api::*;
use cantrip_mechanics::chosen_type::enters_choosing_a_creature_type;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_choosing_a_creature_type),
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        // "Spend this mana only to cast a creature spell of the chosen type or activate an ability of a creature source of the
        // chosen type" (CR 106.6, 205.3m, 602.2b): the type chosen as the land entered, or nothing at all if none was (its
        // ruling: a land that never entered).
        mana_ability(
            TAP,
            add_any_color(1).only_for(ManaRestriction::Subtypes(SubtypeSpells::CHOSEN_TYPE_CREATURE_SPELLS_OR_ABILITIES)),
            "{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type or activate an ability of a creature source of the chosen type.",
        ),
    ],
    ..oracle::SECLUDED_COURTYARD
};
