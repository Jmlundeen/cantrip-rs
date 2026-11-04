use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature gains menace until end of turn. Creatures target player controls get +2/+0 until end of turn." Each part
/// is its own target's and does nothing if that target is no longer legal (CR 608.2b); the creatures pumped are those the player
/// controls as it resolves, the menace creature among them if it is theirs.
fn riot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Menace);
    }
    g.end_sentence();
    effects::creatures_target_player_controls_get::<1, 2, 0>(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE, targets::PLAYER], riot),
    ..oracle::HOW_TO_START_A_RIOT
};
