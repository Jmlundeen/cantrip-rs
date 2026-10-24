use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Exile target creature if it has mana value 3 or less. Corrupted - Exile that creature instead if its controller has
/// three or more poison counters (Corrupted is an ability word, CR 207.2c: it only tags the sentence; CR 122.1f). The
/// mana value and the controller's poison counters are read as the spell resolves (CR 608.2h); the "instead" is text
/// in this one spell, so the creature is exiled once whichever applies (CR 608.2c), and nothing happens at all if it
/// is no longer a legal target (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let corrupted = g.state().player(g.controller(creature)).poison() >= 3;
    if corrupted || g.mana_value(creature) <= 3 {
        g.exile(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::ANOINT_WITH_AFFLICTION
};
