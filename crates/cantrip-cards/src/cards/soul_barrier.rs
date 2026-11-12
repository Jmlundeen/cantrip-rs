use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "This enchantment deals 2 damage to that player unless they pay {2}." (CR 608.2, 120.3, 118.12).
fn damage_that_player_unless_they_pay(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.subject.player() else { return Ok(()) };
    if !g.may_pay(player, item.object, item.object, cost("{2}"), &PAY_QUESTION)? {
        g.damage(item.object, item.subject, 2);
    }
    Ok(())
}

static PAY_QUESTION: Question = Question::new("Pay", "pay {cost}?", "", "Pay {cost}", "Decline");

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, triggers::opponent_casts_creature_spell),
        damage_that_player_unless_they_pay,
        "Whenever an opponent casts a creature spell, this enchantment deals 2 damage to that player unless they pay {2}.",
    )],
    ..oracle::SOUL_BARRIER
};
