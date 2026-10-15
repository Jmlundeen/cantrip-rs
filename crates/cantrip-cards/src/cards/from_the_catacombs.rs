use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::escape;
use cantrip_oracle as oracle;

/// A creature card in any graveyard.
fn creature_card_in_a_graveyard(q: &dyn Query, _: &Asker, t: Target) -> bool {
    cantrip_mechanics::targets::card_in_graveyard(q, t, &[CardType::Creature])
}

/// Put target creature card from a graveyard onto the battlefield under your control with a corpse counter on it. You
/// take the initiative. If that creature would leave the battlefield, exile it instead of putting it anywhere else:
/// an effect of this spell on that one object (CR 611.2c, 614.1a), not an ability it gains.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        let returned = g.put_onto_battlefield(card, item.controller, false)?;
        if g.is_on_battlefield(returned) {
            g.add_counters(returned, CounterKind::Corpse, 1);
            g.exile_on_leave(returned);
        }
    }
    g.take_initiative(item.controller);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: creature_card_in_a_graveyard }], effect }),
    ways: &[escape::way(mana(3, 0, 0, 2, 0, 0), 5, "Escape\u{2014}{3}{B}{B}, Exile five other cards from your graveyard")],
    ..oracle::FROM_THE_CATACOMBS
};
