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
        triggered(triggers::THIS_ENTERS, cantrip_mechanics::effects::take_the_initiative, "When this creature enters, you take the initiative."),
        triggered(triggers::YOUR_END_STEP, grow, "At the beginning of your end step, if you have the initiative, put a +1/+1 counter on this creature.")
            .condition(you_have_the_initiative),
    ],
    ..oracle::PASSAGEWAY_SEER
};
