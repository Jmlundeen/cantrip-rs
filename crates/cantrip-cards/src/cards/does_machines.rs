use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, targets};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn level(q: &dyn Query, source: ObjectId) -> u8 {
    q.obj(source).class_level.max(1)
}

fn at_level_one(q: &dyn Query, asker: &Asker) -> bool {
    level(q, asker.source) == 1
}

fn at_level_two(q: &dyn Query, asker: &Asker) -> bool {
    level(q, asker.source) == 2
}

fn become_level_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.set_class_level(item.object, 2);
    Ok(())
}

fn become_level_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.set_class_level(item.object, 3);
    Ok(())
}

/// "When this Class enters, mill two cards, draw two cards, then discard two cards."
fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 2);
    g.draw(you, 2);
    discard_chosen(g, you, item.object, 2)?;
    Ok(())
}

fn became_level_two(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ClassLevel { object, level: 2 } if *object == source => Fired::PLAIN,
        _ => None,
    }
}

/// "Target artifact cards from your graveyard" (CR 115.1), up to two.
fn artifact_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Artifact])
}

/// "Return up to two target artifact cards from your graveyard to your hand": the ones still legal, together.
fn return_artifacts(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let returning = targets::legal_objects(g, item);
    g.return_to_hand(&returning);
    Ok(())
}

/// "At the beginning of combat on your turn", at level 3 (CR 716.2a): the ability exists only at that level, so it is
/// asked of the level as combat begins, and once triggered it resolves like any other.
fn combat_at_level_three(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::BeginCombat }
            if q.state().turn.active == q.controller(source) && level(q, source) >= 3 =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Target artifact you control" (CR 115.1).
fn artifact_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::ARTIFACT.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

/// "Put three +1/+1 counters on target artifact you control. If it isn't a creature, it becomes a 0/0 Robot creature in
/// addition to its other types." The counters always land; the target is a creature only if it needed to become one, so
/// an artifact creature keeps the power and toughness it had.
fn grow_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(artifact) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(artifact, CounterKind::PlusOne, 3);
    if !g.chars(artifact).is(CardType::Creature) {
        g.add_effect(item.object, artifact, EffectKind::AddTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
        g.add_effect(item.object, artifact, EffectKind::AddSubtype(g.words(item.object).subtype(Subtype::Robot)), Duration::Indefinite);
        g.add_effect(item.object, artifact, EffectKind::SetPt(0, 0), Duration::Indefinite);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{U}: Level 2",
            cost: Cost::mana(mana(1, 0, 1, 0, 0, 0)),
            timing: Timing::Sorcery,
            condition: Some(at_level_one),
            effect: become_level_two,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{4}{U}: Level 3",
            cost: Cost::mana(mana(4, 0, 1, 0, 0, 0)),
            timing: Timing::Sorcery,
            condition: Some(at_level_two),
            effect: become_level_three,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this Class enters, mill two cards, draw two cards, then discard two cards.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: loot,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this Class becomes level 2, return up to two target artifact cards from your graveyard to your hand.",
            listens: EventMask::CLASS_LEVEL,
            matches: became_level_two,
            targets: &[TargetSpec { min: 0, max: 2, filter: artifact_card_in_your_graveyard }],
            effect: return_artifacts,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of combat on your turn, put three +1/+1 counters on target artifact you control. If it isn't a creature, it becomes a 0/0 Robot creature in addition to its other types.",
            listens: EventMask::STEP_BEGAN,
            matches: combat_at_level_three,
            targets: &[TargetSpec { min: 1, max: 1, filter: artifact_you_control }],
            effect: grow_artifact,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::DOES_MACHINES
};
