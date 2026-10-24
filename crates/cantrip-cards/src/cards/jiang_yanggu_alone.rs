use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever a creature you control attacks a player alone" (CR 603.3b, 508.1): the only attacker, attacking a player
/// rather than a planeswalker or battle, and controlled by this ability's controller. The ability is about that attacker.
fn attacks_a_player_alone(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, defender: Target::Player(_), .. }
            if q.state().attackers.len() == 1 && q.controller(*attacker) == q.controller(source) =>
        {
            Fired::about(Target::Object(*attacker))
        }
        _ => None,
    }
}

/// "Discard a card, then draw a card. Then put a +1/+1 counter on that creature for each card you've discarded
/// this turn" Count discards after this discard; draw even if the attacker has left.
fn discard_draw_then_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::discard_then_draw(g, item, 1)?;
    let Target::Object(attacker) = item.subject else { return Ok(()) };
    let discarded = g
        .history()
        .this_turn()
        .iter()
        .filter(|r| matches!(r.event, Event::Discarded { player, .. } if player == item.controller))
        .count() as i16;
    g.add_counters(attacker, CounterKind::PlusOne, discarded);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ATTACKS, attacks_a_player_alone),
        discard_draw_then_grow,
        "Whenever a creature you control attacks a player alone, discard a card, then draw a card. Then put a +1/+1 counter on that creature for each card you've discarded this turn.",
    )],
    ..oracle::JIANG_YANGGU_ALONE
};
