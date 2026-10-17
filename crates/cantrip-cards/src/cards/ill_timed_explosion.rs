use cantrip_api::*;
use cantrip_mechanics::{choose, effects};
use cantrip_oracle as oracle;

const ASK: Question = Question::new(
    "DiscardForSweep",
    "discard two cards?",
    "When you do, it deals damage to each creature equal to the greatest mana value among the cards you discard",
    "Discard two cards",
    "Don't discard",
)
.no_default();

/// "Draw two cards. Then you may discard two cards. When you do, Ill-Timed Explosion deals X damage to each creature, where X is
/// the greatest mana value among cards discarded this way."
///
/// The discard is a choice made as the spell resolves (CR 608.2d), not a cost, and "discard two cards" is only an option with two
/// cards to discard: with fewer in hand it is impossible and not offered, and the reflexive trigger needs that event. Both cards go
/// together (CR 701.9), so what watches discards sees one event. The reflexive ability (CR 603.12) triggers when cards were
/// discarded, carrying X: the greatest mana value among the cards as they are in the graveyard now (CR 202.3: an {X} in a mana cost
/// is 0 there, a land's is 0). The damage is dealt by the spell, as it last existed (CR 608.2h), to every creature at once.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    if hand.len() < 2 || !effects::ask_may(g, you, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    let chosen = choose::cards(g, you, item.object, Purpose::Discard, hand.clone(), 2, false)?;
    // Nothing discarded (nothing a discard can't be made of, CR 101.2) is not doing it: the reflexive ability does not trigger.
    let Some(x) = g.discard_together(&chosen).iter().map(|&card| g.mana_value(card)).max() else { return Ok(()) };
    let this = g.card_id(&CARD);
    g.reflexive_trigger_counted(item.object, you, AbilityRef::delayed(this, 0), Target::None, x as u16);
    Ok(())
}

/// "Ill-Timed Explosion deals X damage to each creature": X is the count the reflexive ability carries.
fn damage_each_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_creature(g, item, item.x as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    delayed: &[delayed_trigger(damage_each_creature, "When you do, Ill-Timed Explosion deals X damage to each creature, where X is the greatest mana value among cards discarded this way.")],
    ..oracle::ILL_TIMED_EXPLOSION
};
