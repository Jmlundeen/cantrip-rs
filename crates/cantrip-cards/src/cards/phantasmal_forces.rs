use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice this creature unless you pay {U}." (CR 608.2, 701.21a, 118.1).
fn sacrifice_creature_unless_pay_u(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{U}"))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            sacrifice_creature_unless_pay_u,
            "At the beginning of your upkeep, sacrifice this creature unless you pay {U}.",
        ),
    ],
    ..oracle::PHANTASMAL_FORCES
};
