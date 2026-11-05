use cantrip_api::*;
use cantrip_mechanics::{combat, targets};
use cantrip_oracle as oracle;

/// The active player may pay the creature's mana value; otherwise it must attack and may be destroyed (CR 118.1, 611.2a).
fn whistle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let payer = g.controller(creature);
    let cost = Cost::mana(generic(g.mana_value(creature)));
    let paid = g.may_pay(payer, item.object, item.object, cost, &PAY_QUESTION)?;
    g.end_sentence();
    if !paid {
        combat::must_attack_and_destroy_if_it_doesnt(g, item, creature);
    }
    Ok(())
}

static PAY_QUESTION: Question = Question::new("Pay", "pay {cost}?", "", "Pay {cost}", "Decline");

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}").and(TAP), whistle, "{3}, {T}: Choose target non-Wall creature the active player has controlled continuously since the beginning of the turn. That player may pay {X}, where X is that creature's mana value. If they don't pay, the creature attacks this turn if able, and at the beginning of the next end step, destroy it if it didn't attack this turn. Activate only before attackers are declared.")
        .targets(&[combat::COMPEL_TARGET]).condition(combat::before_attackers_are_declared)],
    delayed: &[delayed_trigger(combat::destroy_if_it_didnt_attack, "At the beginning of the next end step, destroy it if it didn't attack this turn.")],
    lasting_rules: &[combat::MUST_ATTACK_THIS_TURN],
    ..oracle::ARCUMS_WHISTLE
};
