use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice this creature unless you pay {U}." (CR 608.2, 701.21a, 118.1).
fn sacrifice_creature_unless_pay_u(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    if !g.may_pay(item.controller, item.object, item.object, cost("{U}"), &PAY_QUESTION)? {
        g.sacrifice(item.controller, &[item.object]);
    }
    Ok(())
}

static PAY_QUESTION: Question = Question::new(
    "Pay",
    "pay {cost}?",
    "",
    "Pay {cost}",
    "Decline",
);

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
