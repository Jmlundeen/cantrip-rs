use cantrip_api::*;
use cantrip_mechanics::chosen_type::enters_choosing_a_creature_type;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_choosing_a_creature_type),
    abilities: &[
        // "Spend this mana only to cast a creature spell of the chosen type" (CR 106.6, 205.3m): the type chosen as the
        // artifact entered, or no spell at all if none was.
        mana_ability(
            TAP,
            add_any_color(1).only_for(ManaRestriction::Subtypes(SubtypeSpells::CHOSEN_TYPE_CREATURE)),
            "{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type.",
        ),
    ],
    ..oracle::PILLAR_OF_ORIGINS
};
