use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

const PAY: Question = Question::new("Pay", "pay {cost} to put a +1/+1 counter on this creature?", "", "Pay {cost}", "Don't pay");

/// "You may pay {1}. If you do, put a +1/+1 counter on this creature" (CR 608.2d, 118.12): its controller is asked as the ability
/// resolves, and the counter goes on only if they paid.
fn pay_one_for_a_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, cost("{1}"), &PAY)? {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::ANOTHER_ARTIFACT_YOU_CONTROL_ENTERS,
        pay_one_for_a_counter,
        "Whenever another artifact you control enters, you may pay {1}. If you do, put a +1/+1 counter on this creature.",
    )],
    ..oracle::OREPLATE_PANGOLIN
};
