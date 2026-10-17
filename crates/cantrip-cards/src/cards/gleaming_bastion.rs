use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_one_of(&[Color::White, Color::Blue]), "{T}: Add {W} or {U}. Activate only if this land entered this turn or if you control a basic land.")
            .condition(lands::entered_this_turn_or_you_control_a_basic_land),
    ],
    ..oracle::GLEAMING_BASTION
};
