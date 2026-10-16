use cantrip_api::*;
use cantrip_mechanics::chosen_type::enters_choosing_a_creature_type;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_choosing_a_creature_type),
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        // "Spend this mana only to cast a creature spell of the chosen type, and that spell can't be countered" (CR 106.6,
        // 205.3m, 701.6): a limit and an effect the mana carries. Spent on any cost of the spell - an additional cost
        // such as kicker included, even one paid for a spell cast without paying its mana cost - the spell can't be
        // countered (its rulings).
        mana_ability(
            TAP,
            add_any_color(1).only_for(ManaRestriction::Subtypes(SubtypeSpells::CHOSEN_TYPE_CREATURE.uncounterable())),
            "{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type, and that spell can't be countered.",
        ),
    ],
    ..oracle::CAVERN_OF_SOULS
};
