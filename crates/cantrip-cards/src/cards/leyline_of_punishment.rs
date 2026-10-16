use cantrip_api::*;
use cantrip_mechanics::statics::{damage_cant_be_prevented, players_cant_gain_life};
use cantrip_oracle as oracle;

/// "If this card is in your opening hand" (CR 103.6a): any player may, nothing else is required.
fn in_any_opening_hand(_: &dyn Query, _: &Asker) -> bool {
    true
}

pub static CARD: CardDef = CardDef {
    cant_gain_life: &[players_cant_gain_life],
    unpreventable_damage: &[damage_cant_be_prevented],
    opening_hand: Some(OpeningHandAction { condition: in_any_opening_hand, counter: None, exile_from_hand: 0 }),
    ..oracle::LEYLINE_OF_PUNISHMENT
};
