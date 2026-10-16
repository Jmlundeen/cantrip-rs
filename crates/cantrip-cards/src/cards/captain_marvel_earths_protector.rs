use cantrip_api::*;
use cantrip_mechanics::power_up;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter and an indestructible counter on Captain Marvel." Nothing, if she has left the battlefield
/// since (CR 400.7).
fn power_up_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    g.add_counters(item.object, CounterKind::Indestructible, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{5}{W}{W}"),
        power_up_effect,
        "Power-up \u{2014} {5}{W}{W}: Put a +1/+1 counter and an indestructible counter on Captain Marvel. (Activate each power-up ability only once. Reduce the cost by her mana cost if she entered this turn.)",
    )],
    ..oracle::CAPTAIN_MARVEL_EARTHS_PROTECTOR
};
