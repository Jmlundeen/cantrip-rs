use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

/// Prevent all damage to the target this turn (CR 615.1, 611.2a).
fn resolve(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let target = g.state().target(item, 0);
    prevention::this_turn(g, item, target, 0, Shield::All);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], resolve),
    granted_preventions: &[prevention::ANY_DAMAGE.text(
        "Prevent all damage that would be dealt to target creature this turn.",
    )],
    ..oracle::INDESTRUCTIBLE_AURA
};
