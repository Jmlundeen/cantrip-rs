use cantrip_api::*;
use cantrip_oracle as oracle;

/// This land enters with three mining counters on it.
fn enters_with_mining_counters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Mining as usize] += 3;
    Ok(())
}

/// Add one mana of any color. If there are no mining counters on this land, sacrifice it (CR 605.3a): as the ability
/// resolves, so the mana is added either way.
fn add_one_of_any_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    let land = asker.source;
    if g.is_on_battlefield(land) && g.obj(land).counters(CounterKind::Mining) == 0 {
        g.sacrifice(&[land]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_mining_counters),
    abilities: &[mana_ability(TAP.and(remove_counters(CounterKind::Mining, 1)), add_any_color(1).by(add_one_of_any_color), "{T}, Remove a mining counter from this land: Add one mana of any color. If there are no mining counters on this land, sacrifice it.")],
    ..oracle::GEMSTONE_MINE
};
