use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever an opponent casts a creature spell" (CR 603.2, 601.2i): the spell as it stands on the stack. The ability is about the
/// player who cast it (`StackItem::subject`, "that player").
fn when_opponent_casts_creature_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player != q.controller(source) && q.chars(*spell).is(CardType::Creature) => {
            Fired::about(Target::Player(*player))
        }
        _ => None,
    }
}

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
        when(EventMask::SPELL_CAST, when_opponent_casts_creature_spell),
        damage_that_player_unless_they_pay,
        "Whenever an opponent casts a creature spell, this enchantment deals 2 damage to that player unless they pay {2}.",
    )],
    ..oracle::SOUL_BARRIER
};
