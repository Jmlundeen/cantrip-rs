use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// You get {E}{E}{E} (three energy counters), then you may pay any amount of {E}. Galvanic Discharge deals that much
/// damage to that permanent.
///
/// The three counters are gained first, whatever the target; then any amount of the whole total is offered one counter
/// at a time as a "you may", each yes spending one, until the player stops or has none. Paying nothing is legal, and
/// keeps the counters.
fn discharge(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.change_energy(you, 3);
    let mut paid = 0u16;
    while g.state().player(you).energy > 0 {
        if !cantrip_mechanics::effects::ask_may(g, you, item.object, Purpose::Optional)? {
            break;
        }
        g.change_energy(you, -1);
        paid += 1;
    }
    if paid > 0 {
        damage_target(g, item, 0, paid);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::CREATURE_OR_PLANESWALKER], effect: discharge }),
    ..oracle::GALVANIC_DISCHARGE
};
