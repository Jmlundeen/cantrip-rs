use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice Palladia-Mors unless you pay {R}{G}{W}." (CR 608.2, 701.21a, 118.1).
fn sacrifice_palladia_mors_unless_pay(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{R}{G}{W}"))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            sacrifice_palladia_mors_unless_pay,
            "At the beginning of your upkeep, sacrifice Palladia-Mors unless you pay {R}{G}{W}.",
        ),
    ],
    ..oracle::PALLADIA_MORS
};
