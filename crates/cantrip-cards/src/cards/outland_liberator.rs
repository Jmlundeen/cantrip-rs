use cantrip_api::*;
use cantrip_mechanics::destroy_target;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Target artifact or enchantment defending player controls" (CR 506.2): the player the source attacks, or the
/// controller of the planeswalker it attacks.
fn defending_players_artifact_or_enchantment(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Some(attack) = q.state().attackers.iter().find(|a| a.id == asker.source) else { return false };
    let Some(defending) = q.defending_player(attack.target) else { return false };
    (targets::ARTIFACT_OR_ENCHANTMENT.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == defending)
}

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "{1}, Sacrifice this creature: Destroy target artifact or enchantment." Both faces have it.
const SACRIFICE: AbilityDef = ability(cost("{1}").and(SACRIFICE_THIS), destroy_it, "{1}, Sacrifice this creature: Destroy target artifact or enchantment.")
    .targets(&[targets::ARTIFACT_OR_ENCHANTMENT]);

pub static BACK: CardDef = CardDef {
    abilities: &[
        SACRIFICE,
        triggered(triggers::THIS_ATTACKS, destroy_it, "Whenever this creature attacks, destroy target artifact or enchantment defending player controls.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: defending_players_artifact_or_enchantment, label: "" }]),
    ],
    ..oracle::FRENZIED_TRAPBREAKER
};

pub static CARD: CardDef = CardDef {
    abilities: &[SACRIFICE],
    back: Some(&BACK),
    ..oracle::OUTLAND_LIBERATOR
};
