use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

const PAY: Question = Question::new("Pay", "pay {cost} so another target attacking creature can't be blocked this turn?", "", "Pay {cost}", "Don't pay");

/// "Whenever this Vehicle attacks, you may pay {U}. When you do, another target attacking creature can't be blocked this turn." (CR 603.12):
/// the target is chosen only if the payment is made, as the reflexive ability goes on the stack.
fn pay_for_unblockable(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, cost("{U}"), &PAY)? {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ATTACKS,
            pay_for_unblockable,
            "Whenever this Vehicle attacks, you may pay {U}. When you do, another target attacking creature can't be blocked this turn.",
        ),
        crew(2, "Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    delayed: &[delayed_trigger(
        effects::target_creature_cant_be_blocked_this_turn,
        "When you do, another target attacking creature can't be blocked this turn.",
    )
    .targets(&[targets::ANOTHER_ATTACKING_CREATURE])],
    ..oracle::PASSENGER_FERRY
};
