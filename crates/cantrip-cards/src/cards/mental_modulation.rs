use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast during your turn" (CR 601.2f, 118.7): `player` is the one casting it, and only the
/// generic part of the cost comes down, so on its caster's turn it costs {U}; on an opponent's turn it keeps its full {1}{U}.
fn cheaper_on_your_turn(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    if q.state().turn.active == player {
        CostDelta::less(1)
    } else {
        CostDelta::NONE
    }
}

/// "Tap target artifact or creature. Draw a card." (CR 701.26a, 608.2b): the spell does nothing at all when its only target is
/// illegal as it resolves, so no card is drawn then; a target that is already tapped is still a legal target and the card is
/// drawn.
fn tap_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::tap_targets(g, item)?;
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE], tap_then_draw),
    own_cost_changes: &[cheaper_on_your_turn],
    ..oracle::MENTAL_MODULATION
};
