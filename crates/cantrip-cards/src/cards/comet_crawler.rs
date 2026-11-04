use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, pump, select};
use cantrip_oracle as oracle;

/// "You may sacrifice another creature or artifact. If you do, this creature gets +2/+0 until end of turn." (CR 608.2c-d): the
/// sacrifice is chosen as the ability resolves and may be declined; the pump comes only if a permanent was actually sacrificed.
fn sacrifice_to_get_plus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::may_sacrifice(g, item.controller, item.object, select::another_creature_or_artifact_permanents(select::up_to(1)))?.is_empty() {
        pump(g, item.object, item.object, 2, 0);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        sacrifice_to_get_plus_two,
        "Whenever this creature attacks, you may sacrifice another creature or artifact. If you do, this creature gets +2/+0 until end of turn.",
    )],
    ..oracle::COMET_CRAWLER
};
