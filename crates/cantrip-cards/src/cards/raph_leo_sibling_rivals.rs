use cantrip_api::*;
use cantrip_mechanics::{effects, history::first_combat_phase_of_the_turn, targets, triggers::THIS_ATTACKS};
use cantrip_oracle as oracle;

/// Untap one or two target attacking creatures (CR 701.26a). After this phase, there is an additional combat phase (CR 500.8).
fn untap_and_extra_combat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::untap_targets(g, item)?;
    g.add_combat_phase();
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        untap_and_extra_combat,
        "Whenever Raph & Leo attack, if it's the first combat phase of the turn, untap one or two target attacking creatures. After this phase, there is an additional combat phase.",
    )
    .targets(&[targets::ATTACKING_CREATURE.between(1, 2)])
    .condition(first_combat_phase_of_the_turn)],
    ..oracle::RAPH_LEO_SIBLING_RIVALS
};
