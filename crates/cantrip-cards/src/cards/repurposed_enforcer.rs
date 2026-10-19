use cantrip_api::*;
use cantrip_mechanics::{creatures_you_control, empower, triggers};
use cantrip_oracle as oracle;

/// "Empower Jace X, where X is the number of creatures you control" (CR 701.71a): X is counted as the ability resolves,
/// the Enforcer itself among the creatures (CR 608.2c, 611.2).
fn empower_by_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let n = creatures_you_control(g, item.controller) as i16;
    empower::jace(g, item.controller, item.object, n)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, empower_by_creatures, "Whenever this creature attacks, empower Jace X, where X is the number of creatures you control.")],
    ..oracle::REPURPOSED_ENFORCER
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
