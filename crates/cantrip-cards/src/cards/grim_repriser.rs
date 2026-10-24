use cantrip_api::*;
use cantrip_mechanics::triggers::PROWESS;
use cantrip_oracle as oracle;

/// "Activate only if an opponent has been dealt noncombat damage this turn" (CR 602.5b): this turn's noncombat damage to
/// an opponent of the activating player.
fn opponent_was_dealt_noncombat_damage_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.history().this_turn().iter().any(|r| {
        matches!(r.event, Event::Damage { target: Target::Player(p), amount, kind, .. }
            if !kind.is_combat() && amount > 0 && q.state().opponents(asker.controller).any(|o| o == p))
    })
}

/// "{B}{R}: Return this card from your graveyard to the battlefield with a finality counter on it" (CR 113.6k, 400.7): the
/// card returns as a new object under its owner's control, untapped, with the counter (CR 614.1c).
fn return_with_finality(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let owner = g.obj(item.object).owner;
    g.put_onto_battlefield_with_counters(item.object, owner, false, &[(CounterKind::Finality, 1)])?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        PROWESS,
        ability(
            cost("{B}{R}"),
            return_with_finality,
            "{B}{R}: Return this card from your graveyard to the battlefield with a finality counter on it. Activate only if an opponent has been dealt noncombat damage this turn.",
        )
        .from(ZoneKind::Graveyard)
        .condition(opponent_was_dealt_noncombat_damage_this_turn),
    ],
    ..oracle::GRIM_REPRISER
};
