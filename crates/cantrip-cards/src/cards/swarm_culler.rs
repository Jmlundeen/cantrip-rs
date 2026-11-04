use cantrip_api::*;
use cantrip_mechanics::{effects, select};
use cantrip_mechanics::triggers::THIS_BECOMES_TAPPED;
use cantrip_oracle as oracle;

/// "You may sacrifice another creature or artifact. If you do, draw a card" (CR 608.2c-d): the sacrifice is chosen as the ability
/// resolves and may be declined; the card is drawn only if a permanent was actually sacrificed.
fn sacrifice_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::may_sacrifice(g, item.controller, item.object, select::another_creature_or_artifact_permanents(select::up_to(1)))?.is_empty() {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_BECOMES_TAPPED,
        sacrifice_to_draw,
        "Whenever this creature becomes tapped, you may sacrifice another creature or artifact. If you do, draw a card.",
    )],
    ..oracle::SWARM_CULLER
};
