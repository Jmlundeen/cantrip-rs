use cantrip_api::*;
use cantrip_mechanics::effects::{destroy_first_target, SACRIFICE_TO_DESTROY_ARTIFACT_OR_ENCHANTMENT};
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

pub static BACK: CardDef = CardDef {
    abilities: &[
        SACRIFICE_TO_DESTROY_ARTIFACT_OR_ENCHANTMENT,
        triggered(triggers::THIS_ATTACKS, destroy_first_target, "Whenever this creature attacks, destroy target artifact or enchantment defending player controls.")
            .targets(&[target(defending_players_artifact_or_enchantment)]),
    ],
    ..oracle::FRENZIED_TRAPBREAKER
};

pub static CARD: CardDef = CardDef {
    abilities: &[SACRIFICE_TO_DESTROY_ARTIFACT_OR_ENCHANTMENT],
    back: Some(&BACK),
    ..oracle::OUTLAND_LIBERATOR
};
