use cantrip_api::*;
use cantrip_mechanics::{same_name, targets};
use cantrip_oracle as oracle;

/// Return target nonland permanent and all other permanents with the same name as that permanent to their owners'
/// hands, together. The target first, then the battlefield in its order.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let mut returning = vec![target];
    for &id in g.state().battlefield() {
        if id != target && same_name(g, target, id) {
            returning.push(id);
        }
    }
    g.return_to_hand(&returning);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::NONLAND_PERMANENT], effect }),
    ..oracle::ECHOING_TRUTH
};
