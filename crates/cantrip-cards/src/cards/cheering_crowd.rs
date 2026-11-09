use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("PlusOneCounter", "put a +1/+1 counter on Cheering Crowd?", "If you do, you add {C} for each counter on it", "Put a counter", "Don't");

/// "That player may put a +1/+1 counter on this creature. If they do, they add {C} for each counter on it" (CR 122.1, 106.1, 608.2d):
/// that player decides, and a Crowd that has left the battlefield is a new object that takes no counter (CR 400.7), so there is nothing
/// to choose. The count is of counters of every kind, after the new one is on it.
fn may_put_a_counter_and_add_mana(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(who) = item.subject else { return Ok(()) };
    if !cantrip_mechanics::targets::permanent(g, item.object) || !effects::ask_may(g, who, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    let counters = effects::counters_on(g, item.object);
    g.add_mana(who, Mana::C, counters);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(cantrip_mechanics::triggers::EACH_PLAYERS_FIRST_MAIN_PHASE, may_put_a_counter_and_add_mana, "At the beginning of each player's first main phase, that player may put a +1/+1 counter on this creature. If they do, they add {C} for each counter on it.").produces_mana()],
    ..oracle::CHEERING_CROWD
};
