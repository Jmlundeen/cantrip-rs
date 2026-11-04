use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, destroy this creature unless you pay {3}{B}{B}{B}. If this creature is destroyed this way, it deals 7 damage to you." (CR 608.2c, 120.3, 701.8a).
fn destroy_creature_unless_pay_3(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.may_pay(item.controller, item.object, item.object, cost("{3}{B}{B}{B}"), &PAY_QUESTION)? {
        let destroyed = g.destroy_all(&[item.object]);
        g.end_sentence();
        if !destroyed.is_empty() {
            g.damage(item.object, Target::Player(item.controller), 7);
        }
    }
    Ok(())
}

static PAY_QUESTION: Question = Question::new(
    "Pay",
    "pay {cost}?",
    "If you don't, destroy {subject}",
    "Pay {cost}",
    "Don't pay",
)
.no_default();

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            destroy_creature_unless_pay_3,
            "At the beginning of your upkeep, destroy this creature unless you pay {3}{B}{B}{B}. If this creature is destroyed this way, it deals 7 damage to you.",
        ),
    ],
    ..oracle::COSMIC_HORROR
};
