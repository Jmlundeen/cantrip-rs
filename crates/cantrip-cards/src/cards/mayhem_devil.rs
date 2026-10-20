use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever a player sacrifices a permanent" (CR 701.21a): a permanent that a sacrifice moved from the battlefield under
/// any player's control, this creature's controller's or an opponent's, this creature itself included (CR 603.10a).
fn a_player_sacrifices_a_permanent(q: &dyn Query, _: &Asker, event: &Event) -> Option<Fired> {
    q.state().apnap().find_map(|player| triggers::sacrificed_by(q, player, event)).map(|_| Fired::PLAIN)?
}

/// "This creature deals 1 damage to any target": dealt by the creature, off its last-known information if it has left the
/// battlefield (CR 608.2h).
fn one_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, a_player_sacrifices_a_permanent),
        one_damage,
        "Whenever a player sacrifices a permanent, this creature deals 1 damage to any target.",
    )
    .on_sacrifice()
    .look_back()
    .targets(&[targets::ANY])],
    ..oracle::MAYHEM_DEVIL
};
