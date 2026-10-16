use cantrip_api::*;
use cantrip_mechanics::{
    attach::{attached_object, equip},
    statics::{get_plus_one_plus_one, have_double_strike_and_trample},
};
use cantrip_oracle as oracle;

/// "If this card is in your opening hand, you may begin the game with it on the battlefield" (CR 103.6a): nothing else is
/// required.
fn in_any_opening_hand(_: &dyn Query, _: &Asker) -> bool {
    true
}

pub static CARD: CardDef = CardDef {
    opening_hand: Some(OpeningHandAction { condition: in_any_opening_hand, counter: None, exile_from_hand: 0 }),
    abilities: &[equip(mana("{3}"), "Equip {3}")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, get_plus_one_plus_one, "Equipped creature gets +1/+1."),
        static_ability(Layer::Ability, attached_object, have_double_strike_and_trample, "Equipped creature has double strike and trample."),
    ],
    ..oracle::LEYLINE_AXE
};
