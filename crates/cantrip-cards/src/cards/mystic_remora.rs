use cantrip_api::*;
use cantrip_mechanics::{cumulative_upkeep, cumulative_upkeep::Payment, effects};
use cantrip_oracle as oracle;

/// An opponent cast a noncreature spell; the ability is about its caster (`StackItem::subject`) (CR 603.2).
fn opponent_casts_noncreature(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = event else { return None };
    if q.state().opponents(asker.controller).any(|opponent| opponent == *player) && !q.chars(*spell).is(CardType::Creature) {
        Fired::about(Target::Player(*player))
    } else {
        None
    }
}

const PAY: Question = Question::new("Pay", "pay {cost}, or the ability's controller may draw a card?", "", "Pay {cost}", "Don't pay");

/// The spell's caster may pay first; otherwise the trigger's controller may draw (CR 118.12a, 121.3).
fn draw_unless_paid(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(payer) = item.subject.player() else { return Ok(()) };
    if !g.may_pay(payer, item.object, item.object, cost("{4}"), &PAY)? {
        effects::may_draw_a_card(g, item)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        cumulative_upkeep!(Payment::Mana(mana("{1}")), "Cumulative upkeep {1} (At the beginning of your upkeep, put an age counter on this permanent, then sacrifice it unless you pay its upkeep cost for each age counter on it.)"),
        triggered(when_asked(EventMask::SPELL_CAST, opponent_casts_noncreature), draw_unless_paid,
            "Whenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}."),
    ],
    ..oracle::MYSTIC_REMORA
};
