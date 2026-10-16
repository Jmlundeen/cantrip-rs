use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets, triggers};
use cantrip_oracle as oracle;

/// "Target attacking creature" (CR 115.1, 508.1k).
fn attacking_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.state().attackers.iter().any(|a| a.id == id))
}

/// Until end of turn, the creature gets +2/+2 and gains trample and "Whenever this creature deals combat damage to a
/// player, destroy target artifact that player controls." One effect of the spell, so all three are the same
/// creature's for as long as the turn lasts.
fn grant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::Pump(2, 2), Duration::EndOfTurn);
    g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample])), Duration::EndOfTurn);
    let this = g.obj(item.object).card;
    g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
    Ok(())
}

/// "Target artifact that player controls": the player the creature dealt combat damage to is the one it attacked
/// (CR 506.4, 510.2); the ability is put on the stack with the creature still attacking.
fn artifact_the_attacked_player_controls(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    if !(targets::ARTIFACT.filter)(q, asker, t) {
        return false;
    }
    q.state()
        .attackers
        .iter()
        .any(|a| a.id == asker.source && matches!(a.target, Target::Player(p) if p == q.controller(id)))
}

fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(attacking_creature)], grant),
    delayed: &[triggered(triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT, destroy_artifact, "Whenever this creature deals combat damage to a player, destroy target artifact that player controls.")
        .targets(&[target(artifact_the_attacked_player_controls)])],
    ..oracle::DREADMAWS_IRE
};
