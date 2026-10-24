use cantrip_api::*;
use cantrip_mechanics::{delayed, effects};
use cantrip_oracle as oracle;

/// "Whenever a creature you control deals combat damage to a player or planeswalker": one trigger for each creature and
/// each player or planeswalker it dealt combat damage to (CR 510.2, 603.2c). "You" is the controller of the ability that
/// set this one up (CR 603.7d), so it reads `me.controller`; the dealer is judged as it was when it dealt the damage.
fn creature_you_control_deals_combat_damage_to_player_or_planeswalker(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source, target, amount, kind, .. }
            if kind.is_combat() && *amount > 0 && q.controller(*source) == me.controller && q.chars(*source).is(CardType::Creature) =>
        {
            match target {
                Target::Player(_) => Fired::PLAIN,
                Target::Object(id) if q.chars(*id).is(CardType::Planeswalker) => Fired::PLAIN,
                _ => None,
            }
        }
        _ => None,
    }
}

/// "Whenever a creature you control deals combat damage to a player or planeswalker this turn, draw a card": the
/// ability's resolution sets up a delayed triggered ability with a stated duration (CR 603.7b). Each activation sets up
/// its own, so two activations draw two cards for each creature's damage.
fn watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(effects::enters_with_minus_one_counters::<1>),
    abilities: &[ability(cost("{2}{U}").and(REMOVE_A_COUNTER), watch, "{2}{U}, Remove a counter from this creature: Whenever a creature you control deals combat damage to a player or planeswalker this turn, draw a card.")],
    delayed: &[triggered(
        when_asked(EventMask::DAMAGE, creature_you_control_deals_combat_damage_to_player_or_planeswalker),
        |g, item| effects::draw(g, item, 1),
        "Whenever a creature you control deals combat damage to a player or planeswalker this turn, draw a card.",
    )],
    ..oracle::FLITTERWING_NUISANCE
};
