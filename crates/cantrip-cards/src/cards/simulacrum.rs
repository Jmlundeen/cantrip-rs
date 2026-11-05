use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Damage dealt to this player this turn (CR 120.3).
fn damage_to_player_this_turn(q: &dyn Query, player: PlayerId) -> u32 {
    q.history().this_turn().iter().filter_map(|record| match record.event {
        Event::Damage { target: Target::Player(p), amount, .. } if p == player => Some(amount),
        _ => None,
    }).sum()
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let amount = damage_to_player_this_turn(g, item.controller);
    g.gain_life(item.controller, amount as i32);
    g.end_sentence();
    let amount = damage_to_player_this_turn(g, item.controller);
    g.damage(item.object, g.state().target(item, 0), amount);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    ..oracle::SIMULACRUM
};
