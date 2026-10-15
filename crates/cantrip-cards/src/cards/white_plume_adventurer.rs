use cantrip_api::*;
use cantrip_mechanics::{choose, effects};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "At the beginning of each opponent's upkeep" (CR 503.1).
fn opponents_upkeep(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::Upkeep } if q.state().turn.active != q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// Untap a creature you control, chosen as it resolves and not targeted; if you've completed a dungeon, untap all
/// creatures you control instead (CR 309.7, 608.2d, 701.26). An already untapped creature may be chosen too.
fn untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let creatures: Vec<ObjectId> = g.permanents_of(you).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let chosen = if g.state().player(you).dungeons_completed > 0 {
        creatures
    } else {
        choose::card(g, you, item.object, Purpose::Untap, creatures)?.into_iter().collect()
    };
    for creature in chosen {
        g.untap(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, you take the initiative.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: effects::take_the_initiative,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of each opponent's upkeep, untap a creature you control. If you've completed a dungeon, untap all creatures you control instead.",
            listens: EventMask::STEP_BEGAN,
            matches: opponents_upkeep,
            effect: untap,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::WHITE_PLUME_ADVENTURER
};
