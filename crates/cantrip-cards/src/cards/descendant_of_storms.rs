use cantrip_api::*;
use cantrip_mechanics::{endure, triggers};
use cantrip_oracle as oracle;

const PAY: Question = Question::new("Pay", "pay {cost} so that it endures 1?", "", "Pay {cost}", "Don't pay");

/// "Whenever this creature attacks, you may pay {1}{W}. If you do, it endures 1." (CR 508.1m, 701.63a, 608.2d): the payment is
/// chosen as the ability resolves, and the creature endures only if it was paid.
fn pay_to_endure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, cost("{1}{W}"), &PAY)? {
        endure::endure(g, item.object, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        pay_to_endure,
        "Whenever this creature attacks, you may pay {1}{W}. If you do, it endures 1. (Put a +1/+1 counter on it or create a 1/1 white Spirit creature token.)",
    )],
    ..oracle::DESCENDANT_OF_STORMS
};

pub static EXTRA: &[&CardDef] = &[&endure::SPIRIT];
