use cantrip_api::*;
use cantrip_mechanics::power_up;
use cantrip_oracle as oracle;

/// "Put two +1/+1 counters on this creature."
fn two_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{3}{G}"),
        two_counters,
        "Power-up \u{2014} {3}{G}: Put two +1/+1 counters on this creature. (Activate each power-up ability only once. Reduce the cost by its mana cost if it entered this turn.)",
    )],
    ..oracle::SERPENT_SPECIALIST
};
