use cantrip_api::*;
use cantrip_mechanics::{same_name, targets};
use cantrip_oracle as oracle;

/// "Destroy target nonland permanent and all other permanents with the same name as that permanent." (CR 701.8, 201.2)
/// The target and every other permanent sharing its name are destroyed together (CR 701.8a); the target first, then
/// the battlefield in its order. The spell's only target gone, it does not resolve (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let mut destroying = vec![target];
    for &id in g.state().battlefield() {
        if id != target && same_name(g, target, id) {
            destroying.push(id);
        }
    }
    g.destroy_all(&destroying);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], effect),
    ..oracle::MAELSTROM_PULSE
};
