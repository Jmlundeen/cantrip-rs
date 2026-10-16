use cantrip_api::*;
use cantrip_mechanics::power_up;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on this creature and draw two cards."
fn counter_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    g.draw(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{5}{U}"),
        counter_and_draw,
        "Power-up \u{2014} {5}{U}: Put a +1/+1 counter on this creature and draw two cards. (Activate each power-up ability only once. Reduce the cost by its mana cost if it entered this turn.)",
    )],
    ..oracle::BOLD_BIOCHEMIST
};
