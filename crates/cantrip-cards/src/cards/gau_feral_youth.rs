use cantrip_api::*;
use cantrip_oracle as oracle;

fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

fn end_step(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::End } => Fired::PLAIN,
        _ => None,
    }
}

/// "If a card left your graveyard this turn."
fn card_left(q: &dyn Query, asker: &Asker) -> bool {
    cantrip_mechanics::history::cards_left_graveyard_this_turn(q, asker.controller) > 0
}

/// Gau deals damage equal to its power to each opponent: its power as the ability resolves, or as it last was.
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u16;
    let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(item.controller).map(|p| (item.object, Target::Player(p), power)).collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Rage \u{2014} Whenever Gau attacks, put a +1/+1 counter on it.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::this_attacks,
            effect: counter,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of each end step, if a card left your graveyard this turn, Gau deals damage equal to its power to each opponent.",
            listens: EventMask::STEP_BEGAN,
            matches: end_step,
            condition: Some(card_left),
            effect: damage,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::GAU_FERAL_YOUTH
};
