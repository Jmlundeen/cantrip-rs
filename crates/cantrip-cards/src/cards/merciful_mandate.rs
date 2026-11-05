use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "If it's a combat phase, you may end the combat phase" (CR 724.2g): a choice about the game, not about an object, so
/// there is nothing to target and no object's ability to ask.
const END_THE_COMBAT_PHASE: Question = Question::new(
    "EndTheCombatPhase",
    "End the combat phase?",
    "All attackers and blockers leave combat, every spell and ability on the stack is exiled, and the phase is over",
    "End the combat phase",
    "Let it continue",
);

/// "Untap all creatures you control. If it's a combat phase, you may end the combat phase." The untap is unconditional
/// (CR 701.26a) and comes first; the combat phase ends only as the game is in one as the spell resolves (CR 724.2g), and
/// only if the controller chooses it, the choice taken as the spell resolves (CR 608.2d).
fn untap_and_maybe_end(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::untap_all_creatures_you_control(g, item)?;
    g.end_sentence();
    // The core does nothing outside a combat phase (CR 724.2g), but the question is only asked "if it's a combat phase".
    let in_combat = matches!(
        g.state().turn.step(),
        Step::BeginCombat | Step::DeclareAttackers | Step::DeclareBlockers | Step::CombatDamage | Step::EndOfCombat
    );
    if in_combat
        && g.ask(item.controller, RequestKind::May { source: item.object, purpose: Purpose::Ask(&END_THE_COMBAT_PHASE) })?
            == Answer::Yes
    {
        g.end_the_combat_phase()?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], untap_and_maybe_end),
    ..oracle::MERCIFUL_MANDATE
};
