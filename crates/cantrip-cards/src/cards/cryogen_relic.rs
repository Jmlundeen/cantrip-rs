use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::this_enters, effects};
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
        AbilityDef::Triggered(TriggeredDef {
            text: "When Cryogen Relic enters, draw a card.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: effects::draw::<1>,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Cryogen Relic leaves the battlefield, draw a card.",
            listens: EventMask::ZONE_CHANGE,
            matches: cantrip_mechanics::triggers::this_leaves,
            effect: effects::draw::<1>,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{U}, Sacrifice Cryogen Relic: Put a stun counter on up to one target tapped creature.",
            cost: Cost { mana: mana(1, 0, 1, 0, 0, 0), sacrifice_self: true, ..Cost::NONE },
            targets: &[TargetSpec { min: 0, max: 1, filter: tapped_creature }],
            effect: stun,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::CRYOGEN_RELIC
};
