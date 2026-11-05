use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::CREATURE_DIES};
use cantrip_oracle as oracle;

/// "Whenever a creature dies, put a quest counter on this enchantment. Target opponent loses 1 life. That player loses
/// 2 life instead if there are seven or more quest counters on this enchantment." Adding a counter skips the departed
/// source under the core counter rule; reading its object then uses the last-known count for the threshold (CR 608.2h).
fn trigger(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    // Its only target illegal, the ability does not resolve at all (CR 608.2b).
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    g.add_counters(item.object, CounterKind::Quest, 1);
    let count = g.obj(item.object).counters(CounterKind::Quest);
    g.lose_life(opponent, if count >= 7 { 2 } else { 1 });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(CREATURE_DIES, trigger, "Whenever a creature dies, put a quest counter on this enchantment. Target opponent loses 1 life. That player loses 2 life instead if there are seven or more quest counters on this enchantment.").targets(&[targets::OPPONENT])],
    ..oracle::VENGEANCE_MISSION
};
