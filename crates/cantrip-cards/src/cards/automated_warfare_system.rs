use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, select};
use cantrip_oracle as oracle;

/// "Whenever this creature attacks, you may sacrifice another creature or artifact. If you do, draw a card." The sacrifice
/// is chosen as the ability resolves and may be declined (CR 608.2d, 701.21a); the card is drawn only if a permanent was
/// actually sacrificed (CR 608.2c).
fn sacrifice_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::another_creature_or_artifact_permanents(select::up_to(1));
    if !effects::may_sacrifice(g, item.controller, item.object, rule)?.is_empty() {
        effects::draw(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ATTACKS, sacrifice_to_draw, "Whenever this creature attacks, you may sacrifice another creature or artifact. If you do, draw a card.")],
    ..oracle::AUTOMATED_WARFARE_SYSTEM
};
