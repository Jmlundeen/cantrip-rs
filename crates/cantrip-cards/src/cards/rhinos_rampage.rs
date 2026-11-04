use cantrip_api::*;
use cantrip_mechanics::{effects, fight, targets};
use cantrip_oracle as oracle;

/// "Target noncreature artifact with mana value 3 or less" (CR 115.1, 202.3): an artifact on the battlefield that is not also a
/// creature, as the layers have it, whose mana value is at most 3.
fn noncreature_artifact_with_mana_value_at_most_3(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::NONCREATURE_ARTIFACT.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) <= 3)
}

/// "Target creature you control gets +1/+0 until end of turn. It fights target creature an opponent controls. When excess damage
/// is dealt to the creature an opponent controls this way, destroy up to one target noncreature artifact with mana value 3 or
/// less." The pump is one instruction (CR 608.2b: a target that is illegal on resolution is simply not affected); the fight is
/// the shared verb, which does nothing unless both are creatures on the battlefield (CR 701.14b). Excess damage (CR 120.10) is
/// read from the damage event the fight recorded: damage the creature you control dealt to the other, above its lethal damage.
/// It makes a reflexive ability (CR 603.12), whose own target is chosen as it is put on the stack.
fn pump_fight_and_destroy_an_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::target_creature_gets::<1, 0>(g, item)?;
    let (Some(mine), Some(theirs)) = (targets::object(g, item, 0), targets::object(g, item, 1)) else { return Ok(()) };
    let before = g.history().head() as usize;
    fight::targets_fight(g, item, 0, 1);
    let excess = g.history().all()[before..].iter().any(|r| {
        matches!(r.event, Event::Damage { source, target, recipient_excess, .. }
            if source == mine && target == Target::Object(theirs) && recipient_excess > 0)
    });
    if excess {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS], pump_fight_and_destroy_an_artifact),
    delayed: &[delayed_trigger(effects::destroy_first_target, "When excess damage is dealt to the creature an opponent controls this way, destroy up to one target noncreature artifact with mana value 3 or less.")
        .targets(&[target(noncreature_artifact_with_mana_value_at_most_3).optional()])],
    ..oracle::RHINOS_RAMPAGE
};
