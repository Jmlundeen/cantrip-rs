use cantrip_api::*;
use cantrip_mechanics::mana::TAP_FOR_ANY_COLOR;
use cantrip_oracle as oracle;

/// Storing mana adds none, so this is an ordinary activated ability (CR 605.1a): it uses the stack.
fn store(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Charge, 1);
    Ok(())
}

/// "At the beginning of your precombat main phase" (CR 603.2b).
fn your_precombat_main(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::PrecombatMain } if q.state().turn.active == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// Every charge counter comes off at once, counted as the ability resolves, and each adds one mana of a color its
/// controller chooses (the same color may be chosen again). A Relic that has left removes nothing.
fn release(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let relic = item.object;
    let all = g.obj(relic).counters(CounterKind::Charge);
    let released = g.remove_counters(relic, CounterKind::Charge, all);
    if released <= 0 {
        return Ok(());
    }
    let ask = RequestKind::ChooseColors { source: relic, choices: ColorSet(0b11111), count: released as u8 };
    if let Answer::Colors(colors) = g.ask(item.controller, ask)? {
        for color in colors {
            g.add_mana(item.controller, color.mana(), 1);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        TAP_FOR_ANY_COLOR,
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Put a charge counter on Coalition Relic.",
            cost: Cost { tap: true, ..Cost::NONE },
            effect: store,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your precombat main phase, remove all charge counters from Coalition Relic. Add one mana of any color for each charge counter removed this way.",
            listens: EventMask::STEP_BEGAN,
            matches: your_precombat_main,
            effect: release,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::COALITION_RELIC
};
