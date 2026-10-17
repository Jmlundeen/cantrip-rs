use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("PayEnergyCost", "pay three energy?", "When you do, put two +1/+1 counters and a flying counter on target attacking creature, and it becomes an Angel", "Pay 3 energy", "Don't pay").amount(3);

/// You gain 1 life and get {E} (an energy counter).
fn bank(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    g.change_energy(item.controller, 1);
    Ok(())
}

/// You may pay {E}{E}{E}. When you do (CR 603.12), the reflexive trigger follows. The payment is made as the ability
/// resolves, so it is an optional payment in the effect, and a player without all three is not asked.
fn pay_energy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if g.state().player(you).energy < 3 {
        return Ok(());
    }
    if !cantrip_mechanics::effects::ask_may(g, you, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    g.change_energy(you, -3);
    let this = g.card_id(&CARD);
    g.reflexive_trigger(item.object, you, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

/// "Target attacking creature" (CR 115.1, 506.4): declared as an attacker and still attacking.
fn attacking_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.state().attackers.iter().any(|a| a.id == id))
}

/// Put two +1/+1 counters and a flying counter on target attacking creature. It becomes an Angel in addition to its
/// other types, with no duration (CR 205.1b, 611.2a).
fn empower(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 2);
    g.add_counters(creature, CounterKind::Flying, 1);
    g.add_effect(item.object, creature, EffectKind::AddSubtype(g.words(item.object).subtype(Subtype::Angel)), Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS, bank, "Whenever another creature you control enters, you gain 1 life and get {E} (an energy counter)."),
        triggered(cantrip_mechanics::triggers::YOU_ATTACK, pay_energy, "Whenever you attack, you may pay {E}{E}{E}. When you do, put two +1/+1 counters and a flying counter on target attacking creature. It becomes an Angel in addition to its other types."),
    ],
    delayed: &[delayed_trigger(empower, "When you do, put two +1/+1 counters and a flying counter on target attacking creature. It becomes an Angel in addition to its other types.")
        .targets(&[target(attacking_creature)])],
    ..oracle::GUIDE_OF_SOULS
};
