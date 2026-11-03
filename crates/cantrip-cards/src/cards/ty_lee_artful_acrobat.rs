use cantrip_api::*;
use cantrip_mechanics::triggers::{PROWESS, THIS_ATTACKS};
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

const PAY: Question = Question::new(
    "PayForBlockRestriction",
    "pay {cost} so target creature can't block this turn?",
    "When you do, target creature can't block this turn",
    "Pay {cost}",
    "Don't pay",
);

/// "Whenever Ty Lee attacks, you may pay {1}. When you do, ..." (CR 118.12, 603.12): paying is doing it, and the reflexive ability
/// triggers then, its target chosen as it is put on the stack. Declining pays nothing and triggers nothing.
fn pay_to_stop_a_blocker(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, cost("{1}"), &PAY)? {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        PROWESS,
        triggered(
            THIS_ATTACKS,
            pay_to_stop_a_blocker,
            "Whenever Ty Lee attacks, you may pay {1}. When you do, target creature can't block this turn.",
        ),
    ],
    delayed: &[delayed_trigger(effects::first_target_cant_block_this_turn, "When you do, target creature can't block this turn.")
        .targets(&[targets::CREATURE])],
    ..oracle::TY_LEE_ARTFUL_ACROBAT
};
