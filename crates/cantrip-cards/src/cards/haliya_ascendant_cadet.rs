use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "A creature you control with a +1/+1 counter on it deals combat damage to a player": one of the batch that triggers the second
/// ability (CR 510.2, 603.2c). The counter must be on the creature when the damage is dealt (the 2025-07-25 ruling), which is when
/// this is asked, as the damage happens; the player damaged is the trigger's subject.
fn creature_with_a_counter_deals_combat_damage_to_player(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Player(p), kind, .. }
            if kind.is_combat()
                && q.controller(*from) == me.controller
                && q.chars(*from).is(CardType::Creature)
                && q.obj(*from).counters(CounterKind::PlusOne) > 0 =>
        {
            Fired::about(Target::Player(*p))
        }
        _ => None,
    }
}

/// "Draw a card."
fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS_OR_ATTACKS,
            effects::plus_one_counter_on_target,
            "Whenever Haliya enters or attacks, put a +1/+1 counter on target creature you control.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL]),
        // "One or more creatures ... deal combat damage to a player" (CR 603.2c): one trigger for each player the batch damaged,
        // however many of the creatures dealt it.
        triggered(
            when_asked(EventMask::DAMAGE, creature_with_a_counter_deals_combat_damage_to_player),
            draw,
            "Whenever one or more creatures you control with +1/+1 counters on them deal combat damage to a player, draw a card.",
        )
        .sums_per_subject(),
    ],
    ..oracle::HALIYA_ASCENDANT_CADET
};
