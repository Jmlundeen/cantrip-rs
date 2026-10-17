use cantrip_api::*;
use cantrip_mechanics::lands::entered_this_turn_or_you_control_a_basic_land;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(
            TAP,
            add_one_of(&[Color::Red, Color::Green]),
            "{T}: Add {R} or {G}. Activate only if this land entered this turn or if you control a basic land.",
        )
        .condition(entered_this_turn_or_you_control_a_basic_land),
    ],
    ..oracle::TRAINING_COMPOUND
};
