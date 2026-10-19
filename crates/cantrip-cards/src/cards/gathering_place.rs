use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_one_of(&[Color::Green, Color::White]), "{T}: Add {G} or {W}. Activate only if this land entered this turn or if you control a basic land.")
            .condition(lands::entered_this_turn_or_you_control_a_basic_land),
    ],
    ..oracle::GATHERING_PLACE
};
