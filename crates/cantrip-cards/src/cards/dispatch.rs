use cantrip_api::*;
use cantrip_mechanics::{artifacts_you_control, targets};
use cantrip_oracle as oracle;

/// "Tap target creature. Metalcraft — If you control three or more artifacts, exile that creature." (CR 701.26a,
/// 608.2c): the creature is tapped first, then the artifact count is read as the spell resolves, so an artifact that
/// left in response changes it (CR 608.2h).
fn tap_and_maybe_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.tap(creature);
    if artifacts_you_control(g, item.controller) >= 3 {
        g.exile(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], tap_and_maybe_exile),
    ..oracle::DISPATCH
};
