use cantrip_api::*;
use cantrip_mechanics::{choose, effects, targets, triggers};
use cantrip_oracle as oracle;

const SULTAI: u8 = 0;
const ABZAN: u8 = 1;

/// "As this enchantment enters, choose Sultai or Abzan" (CR 614.12a, 607.2f).
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    choose::option_as_enters(g, entering, &CARD, 0..2)
}

/// "• Sultai — Whenever a counter is put on a creature you control" (CR 122.6, 603.2): counters of any kind, put by anyone, on a
/// creature this ability's controller controls. Only while Sultai was chosen.
fn counter_put_on_a_creature_you_control(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, .. }
            if choose::chosen_option(q, me, &CARD) == Some(SULTAI)
                && q.is_on_battlefield(*object)
                && q.controller(*object) == me.controller
                && q.chars(*object).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "• Abzan — Whenever you attack" (CR 508.1m): only while Abzan was chosen.
fn you_attack_while_abzan(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    (choose::chosen_option(q, me, &CARD) == Some(ABZAN)).then(|| triggers::you_attack(q, me, event)).flatten()
}

/// "Put a +1/+1 counter on target attacking creature. It gains menace until end of turn" (CR 122.1, 611.2a).
fn counter_and_menace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Menace);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    labels: &["Sultai", "Abzan"],
    abilities: &[
        triggered(
            when_asked(EventMask::COUNTERS, counter_put_on_a_creature_you_control),
            |g, item| effects::draw(g, item, 1),
            "Sultai \u{2014} Whenever a counter is put on a creature you control, draw a card. This ability triggers only once each turn.",
        )
        .triggers_each_turn(1),
        triggered(
            when_asked(EventMask::ATTACKERS_DECLARED, you_attack_while_abzan),
            counter_and_menace,
            "Abzan \u{2014} Whenever you attack, put a +1/+1 counter on target attacking creature. It gains menace until end of turn.",
        )
        .targets(&[targets::ATTACKING_CREATURE]),
    ],
    ..oracle::HOLLOWMURK_SIEGE
};
