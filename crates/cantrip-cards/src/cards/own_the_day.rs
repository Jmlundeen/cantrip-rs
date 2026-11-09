use cantrip_api::*;
use cantrip_mechanics::{battlefront, effects};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast for each opponent you attacked this turn" (CR 601.2f, 118.7a, 508.1):
/// Battlefront's shared distinct-player history count, applied to generic mana only.
fn cheaper(q: &dyn Query, _: ObjectId, you: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(battlefront::opponents_attacked_this_turn(q, you) as u32)
}

/// "Untap all creatures that attacked this turn" (CR 701.26b, 508.1, 400.7): current creatures on either side,
/// declared attacking this turn; a returned creature is a new object and was not that attacker.
/// "After this main phase, there is an additional combat phase followed by an additional main phase" (CR 500.8):
/// after this individual main phase, and only while a main phase is underway.
fn untap_and_add_phases(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    effects::untap_creatures_that_attacked_this_turn(g);
    g.end_sentence();
    if g.state().turn.step().is_main() {
        g.add_phases(PhaseAnchor::Current, &[PhaseSpec::Whole(Phase::Combat), PhaseSpec::Whole(Phase::Main)]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[cheaper],
    spell: spell(&[], untap_and_add_phases),
    ..oracle::OWN_THE_DAY
};
