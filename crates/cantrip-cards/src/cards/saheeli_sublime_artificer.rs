use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::tokens::SERVO;
use cantrip_mechanics::triggers::noncreature_spell_you_cast;
use cantrip_oracle as oracle;

fn make_servo(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &SERVO, 1)?;
    Ok(())
}

fn artifact_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && q.controller(id) == asker.controller && q.chars(id).is(CardType::Artifact))
}

/// "Another target artifact or creature you control": not the permanent the first target slot chose.
fn other_artifact_or_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && q.controller(id) == asker.controller
        && { let c = q.chars(id); c.is(CardType::Artifact) || c.is(CardType::Creature) }
        && !q.chosen_targets().contains(&t))
}

/// Target artifact you control becomes a copy of another target artifact or creature you control until end of turn, except it's
/// an artifact in addition to its other types (CR 707.9b). Both targets must still be legal.
fn become_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.target_legal(item, 0) || !g.target_legal(item, 1) {
        return Ok(());
    }
    let (Target::Object(artifact), Target::Object(copied)) = (item.targets[0], item.targets[1]) else { return Ok(()) };
    let values = g.copiable_values(copied).with_type(CardType::Artifact);
    g.become_copy(item.object, artifact, values);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you cast a noncreature spell, create a 1/1 colorless Servo artifact creature token.",
            listens: EventMask::SPELL_CAST,
            matches: noncreature_spell_you_cast,
            effect: make_servo,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}2: Target artifact you control becomes a copy of another target artifact or creature you control until end of turn, except it's an artifact in addition to its other types.",
            cost: Cost { loyalty: Some(-2), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[
                TargetSpec { min: 1, max: 1, filter: artifact_you_control },
                TargetSpec { min: 1, max: 1, filter: other_artifact_or_creature_you_control },
            ],
            effect: become_copy,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::SAHEELI_SUBLIME_ARTIFICER
};

pub static EXTRA: &[&CardDef] = &[&SERVO];
