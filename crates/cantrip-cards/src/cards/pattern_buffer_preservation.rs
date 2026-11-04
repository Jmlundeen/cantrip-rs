use cantrip_api::*;
use cantrip_mechanics::{beam_me_up, library::look_at_top_take_one_rest_to_graveyard};
use cantrip_oracle as oracle;

/// "Look at the top two cards of your library. Put one of them into your hand and the other into your graveyard."
/// (CR 401.2, 609.3)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top_take_one_rest_to_graveyard(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[beam_me_up::way(mana("{1}{B}"), "Beam me up {1}{B}")],
    ..oracle::PATTERN_BUFFER_PRESERVATION
};
