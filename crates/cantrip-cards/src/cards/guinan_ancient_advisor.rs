use cantrip_api::*;
use cantrip_mechanics::history::entered_this_turn;
use cantrip_oracle as oracle;

/// "{1}, {T}: Put a +1/+1 counter on each creature you control that entered this turn. If you put two or more +1/+1
/// counters on creatures this way, draw a card." (CR 603.6a, 608.2c): the creatures are chosen as the ability resolves,
/// the counters go on all of them at once, and the card is drawn only when that was two or more.
fn counter_newcomers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let newcomers: Vec<ObjectId> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| g.chars(id).is(CardType::Creature) && entered_this_turn(g.history(), id))
        .collect();
    let count = newcomers.len();
    instruction(g, |g| {
        for &creature in &newcomers {
            g.add_counters(creature, CounterKind::PlusOne, 1);
        }
    });
    if count >= 2 {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}, {T}"),
        counter_newcomers,
        "{1}, {T}: Put a +1/+1 counter on each creature you control that entered this turn. If you put two or more +1/+1 counters on creatures this way, draw a card.",
    )],
    ..oracle::GUINAN_ANCIENT_ADVISOR
};
