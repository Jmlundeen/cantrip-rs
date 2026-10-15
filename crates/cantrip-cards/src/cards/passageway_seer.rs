use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "If you have the initiative" (CR 603.4): asked as the ability would trigger and again as it resolves.
fn you_have_the_initiative(q: &dyn Query, asker: &Asker) -> bool {
    q.state().player(asker.controller).initiative
}

/// Put a +1/+1 counter on this creature, if it is still the permanent that triggered.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, you take the initiative.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            effect: cantrip_mechanics::effects::take_the_initiative,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your end step, if you have the initiative, put a +1/+1 counter on this creature.",
            listens: EventMask::STEP_BEGAN,
            matches: triggers::your_end_step,
            condition: Some(you_have_the_initiative),
            effect: grow,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::PASSAGEWAY_SEER
};
