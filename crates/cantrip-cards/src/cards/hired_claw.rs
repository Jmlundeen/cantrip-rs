use cantrip_api::*;
use cantrip_mechanics::effects::{self, plus_one_counter_on_this};
use cantrip_mechanics::{history::an_opponent_lost_life, targets};
use cantrip_oracle as oracle;

/// "Whenever you attack with one or more Lizards" (CR 508.1m, 603.2c): a creature its controller declares as an attacker
/// that is a Lizard, the type read through the words of this creature's text (CR 612). The attackers declared together are
/// one batch, so the ability triggers once however many Lizards attack.
fn you_attack_with_a_lizard(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. }
            if q.controller(*attacker) == me.controller
                && q.chars(*attacker).has_subtype(q.words(me.source).subtype(Subtype::Lizard)) =>
        {
            Fired::about(Target::Object(*attacker))
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::ATTACKS, you_attack_with_a_lizard), effects::damage_first_target::<1>, "Whenever you attack with one or more Lizards, this creature deals 1 damage to target opponent.")
            .targets(&[targets::OPPONENT])
            .once_per_batch(),
        ability(cost("{1}{R}"), plus_one_counter_on_this, "{1}{R}: Put a +1/+1 counter on this creature. Activate only if an opponent lost life this turn and only once each turn.")
            .condition(an_opponent_lost_life)
            .once_each_turn(),
    ],
    ..oracle::HIRED_CLAW
};
