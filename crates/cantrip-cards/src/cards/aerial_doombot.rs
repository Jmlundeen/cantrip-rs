use cantrip_api::*;
use cantrip_mechanics::power_up;
use cantrip_oracle as oracle;

/// "Put three +1/+1 counters on this creature."
fn three_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{5}{U}"),
        three_counters,
        "Power-up \u{2014} {5}{U}: Put three +1/+1 counters on this creature. (Activate each power-up ability only once. Reduce the cost by its mana cost if it entered this turn.)",
    )],
    ..oracle::AERIAL_DOOMBOT
};
