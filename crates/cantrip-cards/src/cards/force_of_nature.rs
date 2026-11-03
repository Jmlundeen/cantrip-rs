use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

const PAY: Question = Question::new("Pay", "pay {cost}, or {subject} deals 8 damage to you?", "", "Pay {cost}", "Take 8 damage");

/// "At the beginning of your upkeep, this creature deals 8 damage to you unless you pay {G}{G}{G}{G}." (CR 608.2, 120.3, 118.12).
fn deals_8_damage_unless_paid(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.may_pay(item.controller, item.object, item.object, cost("{G}{G}{G}{G}"), &PAY)? {
        g.damage(item.object, Target::Player(item.controller), 8);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            deals_8_damage_unless_paid,
            "At the beginning of your upkeep, this creature deals 8 damage to you unless you pay {G}{G}{G}{G}.",
        ),
    ],
    ..oracle::FORCE_OF_NATURE
};
