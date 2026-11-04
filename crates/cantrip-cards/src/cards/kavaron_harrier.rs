use cantrip_api::*;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::tokens::ROBOT_2_2;
use cantrip_mechanics::{delayed, triggers};
use cantrip_oracle as oracle;

const PAY: Question = Question::new("Pay", "pay {cost} to create a 2/2 Robot token tapped and attacking?", "", "Pay {cost}", "Don't pay");

/// "Whenever this creature attacks, you may pay {2}. If you do, create a 2/2 colorless Robot artifact creature token that's
/// tapped and attacking. Sacrifice that token at end of combat." The payment is chosen as the ability resolves; the token
/// attacks what its controller chooses (the rulings, CR 508.4) and is never declared as an attacker, so nothing that triggers
/// on attackers sees it. The delayed sacrifice (CR 603.7) is about the token made, whatever a replacement made of it.
fn pay_for_a_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.may_pay(item.controller, item.object, item.object, cost("{2}"), &PAY)? {
        return Ok(());
    }
    let robot = g.card_id(&ROBOT_2_2);
    let made = create_attacking_of_your_choice(g, item.controller, item.object, robot, 1, true)?;
    if !made.is_empty() {
        g.delayed_trigger_about(
            DelayedWhen::Step { step: Step::EndOfCombat, on_turn_of: None },
            item.object,
            item.controller,
            AbilityRef::delayed(item.ability.card, 0),
            &made,
        );
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        pay_for_a_robot,
        "Whenever this creature attacks, you may pay {2}. If you do, create a 2/2 colorless Robot artifact creature token that's tapped and attacking. Sacrifice that token at end of combat.",
    )],
    delayed: &[delayed_trigger(delayed::sacrifice_created, "Sacrifice that token at end of combat.")],
    ..oracle::KAVARON_HARRIER
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
