use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS, effects};
use cantrip_oracle as oracle;

/// "Target tapped creature" (CR 115.1): tapped when chosen and, legality being checked again on resolution, still
/// tapped then.
fn tapped_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.obj(id).tapped)
}

/// Put a stun counter on up to one target tapped creature.
fn stun(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::Stun, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When Cryogen Relic enters, draw a card."),
        triggered(cantrip_mechanics::triggers::THIS_LEAVES, |g, item| effects::draw(g, item, 1), "When Cryogen Relic leaves the battlefield, draw a card.")
            .look_back(),
        ability(cost("{1}{U}").and(SACRIFICE_THIS), stun, "{1}{U}, Sacrifice Cryogen Relic: Put a stun counter on up to one target tapped creature.")
            .targets(&[up_to_one(tapped_creature)]),
    ],
    ..oracle::CRYOGEN_RELIC
};
