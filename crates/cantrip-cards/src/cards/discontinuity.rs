use cantrip_api::*;
use cantrip_oracle as oracle;

/// "During your turn, this spell costs {2}{U}{U} less to cast." (CR 601.2f, 118.7): `player` is the one casting it; the
/// two blue come off its blue symbols (118.7b), so on its caster's turn it costs {1}{U}.
fn cheaper_on_your_turn(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed) -> CostDelta {
    if matches!(costed, Costed::Spell(_)) && q.state().turn.active == player {
        CostDelta::less_mana(mana("{2}{U}{U}"))
    } else {
        CostDelta::NONE
    }
}

/// "End the turn." (CR 724.1): Discontinuity is exiled from the stack with every other spell and ability on it (724.1b).
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    g.end_the_turn()
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    own_cost_changes: &[cheaper_on_your_turn],
    ..oracle::DISCONTINUITY
};
