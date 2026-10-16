use cantrip_api::*;
use cantrip_mechanics::{same_name, targets};
use cantrip_oracle as oracle;

/// Destroy target artifact and all other artifacts with the same name as that artifact, together. The target first,
/// then the battlefield in its order.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let mut destroying = vec![target];
    for &id in g.state().battlefield() {
        if id != target && g.chars(id).is(CardType::Artifact) && same_name(g, target, id) {
            destroying.push(id);
        }
    }
    g.destroy_all(&destroying);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT], effect),
    ..oracle::ECHOING_RUIN
};
