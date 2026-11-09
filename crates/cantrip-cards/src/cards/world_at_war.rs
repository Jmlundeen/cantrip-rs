use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "After the second main phase this turn, there's an additional combat phase followed by an additional main
/// phase" (CR 500.8, 505.1a-b): the first postcombat main, whether future or current; a completed anchor adds nothing.
/// "At the beginning of that combat" (CR 603.7a-b): only the individual combat returned by this insertion.
fn add_phases(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let phases = g.add_phases(PhaseAnchor::Main(2), &[PhaseSpec::Whole(Phase::Combat), PhaseSpec::Whole(Phase::Main)]);
    if let Some(&phase) = phases.first() {
        g.delayed_trigger(
            DelayedWhen::PhaseStep { step: Step::BeginCombat, phase },
            item.object,
            item.controller,
            AbilityRef::delayed(item.ability.card, 0),
            Target::None,
        );
    }
    Ok(())
}

/// "Untap all creatures that attacked this turn" (CR 701.26b, 508.1, 400.7): query attack history when the delayed
/// ability resolves, across controllers, retaining exact object identity. Own the Day uses the same instruction.
fn untap(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    effects::untap_creatures_that_attacked_this_turn(g);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], add_phases),
    delayed: &[delayed_trigger(untap, "At the beginning of that combat, untap all creatures that attacked this turn.")],
    rebound: true,
    ..oracle::WORLD_AT_WAR
};
