use cantrip_api::*;
use cantrip_oracle as oracle;

/// Storing mana adds none, so this is an ordinary activated ability (CR 605.1a): it uses the stack.
fn store(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Charge, 1);
    Ok(())
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
    for color in cantrip_mechanics::choose_colors(g, item.controller, relic, ColorSet(0b11111), released as u8)? {
        g.add_mana(item.controller, color.mana(), 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(TAP, store, "{T}: Put a charge counter on this artifact."),
        triggered(cantrip_mechanics::triggers::YOUR_FIRST_MAIN_PHASE, release, "At the beginning of your first main phase, remove all charge counters from this artifact. Add one mana of any color for each charge counter removed this way."),
    ],
    ..oracle::COALITION_RELIC
};
