use cantrip_api::*;
use cantrip_mechanics::{choose, select};
use cantrip_oracle as oracle;

/// "Each player puts a vow counter on a creature they control and sacrifices the rest."
/// Choices are mandatory if there is a creature, in APNAP order before any counters are put
/// (CR 101.4, 608.2d-e). Each player puts their own counters, simultaneously, then sacrifices
/// their other creatures simultaneously (CR 122.6, 701.21a). No targeting is involved.
fn promise(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut kept = Vec::new();
    let mut rest = Vec::new();
    for player in g.state().apnap().collect::<Vec<_>>() {
        let creatures: Vec<_> = g.state().battlefield().iter().copied()
            .filter(|&id| g.controller(id) == player && g.chars(id).is(CardType::Creature)).collect();
        let chosen = choose::select(g, player, item.object, Purpose::Choose, creatures.clone(), select::creature_cards(select::as_many_as_can(1)))?;
        rest.extend(creatures.into_iter().filter(|id| !chosen.contains(id)).map(|id| (player, id)));
        kept.extend(chosen.into_iter().map(|id| (player, id, CounterKind::Vow, 1)));
    }
    g.add_counters_together_by(&kept);
    g.sacrifice_each(&rest);
    g.end_sentence();
    // "Each of those creatures can't attack you or planeswalkers you control for as long as it
    // has a vow counter on it." The object identities and "you" stay fixed (CR 400.7, 611.2c).
    // WhileCounter ends permanently at zero, even within one resolution (CR 611.2b).
    cantrip_api::instruction(g, |g| {
        for (_, id, _, _) in kept {
            g.add_effect(
                item.object,
                id,
                EffectKind::CantAttackPlayer { player: item.controller, planeswalkers: true },
                Duration::WhileCounter(CounterKind::Vow),
            );
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], promise), ..oracle::PROMISE_OF_LOYALTY };
