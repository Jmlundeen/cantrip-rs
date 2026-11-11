use cantrip_api::*;
use cantrip_mechanics::{assimilate, effects, targets};
use cantrip_oracle as oracle;

/// "Target nonartifact creature an opponent controls" (CR 115.1, 205.2): a creature on the battlefield that is not an
/// artifact as the layers have it now, and whose controller is not this ability's controller. An assimilated creature is an
/// artifact, so a later chapter I cannot take it a second time.
fn nonartifact_creature_an_opponent_controls(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if targets::permanent(q, id)
            && q.chars(id).is(CardType::Creature)
            && !q.chars(id).is(CardType::Artifact)
            && q.controller(id) != asker.controller)
}

/// I: "Assimilate target nonartifact creature an opponent controls." (CR 613.1b, 613.1d, 613.4, 108.4, 122.6): the creature
/// is under this Saga's controller's control with no end, an artifact creature that is a Borg and no longer any other
/// creature type, with a +1/+1 counter. Chapter III, a linked ability of the same source (CR
/// 607.2d), finds it in the event history, and nothing happens if the target is no longer legal (CR 608.2b).
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    assimilate::assimilate_permanent(g, item, creature);
    Ok(())
}

/// The creatures this Saga's chapter I has assimilated, once for each, however many times that ability resolved (CR 607.3: a
/// copy of it acted too): the permanents whose control an ability of `saga` changed, oldest first.
fn assimilated_with(q: &dyn Query, saga: ObjectId) -> Vec<ObjectId> {
    let mut borg: Vec<ObjectId> = Vec::new();
    for r in q.history().select(EventQuery::kind(EventMask::CONTROL_CHANGED)) {
        if let (Cause::Ability { source, .. }, Event::ControlChanged { permanent, .. }) = (r.cause, r.event)
            && source == saga
            && !borg.contains(&permanent)
        {
            borg.push(permanent);
        }
    }
    borg
}

/// II: "Artifact creatures you control get +2/+2 until end of turn." (CR 611.2c): the artifact creatures this chapter's
/// controller has as it resolves, and no creature that becomes an artifact creature later.
fn chapter_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    effects::pump_creatures_where(g, item.object, |g, id| g.controller(id) == you && g.chars(id).is(CardType::Artifact), 2, 2);
    Ok(())
}

/// III: "Exile the Borg assimilated with this Saga, then return it to the battlefield under its owner's control." (CR 607.2d,
/// 400.7, 610.3): the creature chapter I assimilated, if it is still that object on the battlefield - one that left and came
/// back is a new object none of this reaches. It returns as a new object with its printed characteristics, so the counter and
/// the Borg types are gone; a token assimilated this way ceases to exist and does not return (CR 111.7).
fn chapter_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut assimilated = assimilated_with(g, item.object);
    assimilated.retain(|&id| g.is_on_battlefield(id));
    cantrip_mechanics::exile_and_return(g, &assimilated)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, chapter_one, "I \u{2014} Assimilate target nonartifact creature an opponent controls.")
            .targets(&[target(nonartifact_creature_an_opponent_controls)]),
        chapter(2, chapter_two, "II \u{2014} Artifact creatures you control get +2/+2 until end of turn."),
        chapter(
            3,
            chapter_three,
            "III \u{2014} Exile the Borg assimilated with this Saga, then return it to the battlefield under its owner's control.",
        ),
    ],
    ..oracle::THE_BEST_OF_BOTH_WORLDS
};
