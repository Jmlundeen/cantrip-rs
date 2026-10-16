use cantrip_api::*;
use cantrip_mechanics::top_of_library;
use cantrip_oracle as oracle;

/// "Whenever a creature you control attacks" (CR 508.3a, 603.2c): declaring attackers names each attacker in its own
/// event, so the ability triggers once for each creature of its controller's that attacks.
fn creature_you_control_attacks(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if q.controller(*attacker) == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// "Exile the top card of your library face down. (You can't look at it.)" (CR 406.3): linked to this enchantment
/// (CR 607.2a), so its last ability finds it. Nobody looks at it: it came off a library.
fn exile_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(&card) = top_of_library(g, item.controller, 1).first() {
        g.exile_face_down(&[card], Some(item.object));
    }
    Ok(())
}

/// "Put all cards exiled with this enchantment into their owners' hands." (CR 607.2a.) The cost sacrificed the
/// enchantment, so the cards are found off the object the ability came from (CR 113.7a), and they go together.
fn return_exiled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled = g.exiled_with(item.object);
    g.return_to_hand(&exiled);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ATTACKS, creature_you_control_attacks), exile_top, "Whenever a creature you control attacks, exile the top card of your library face down. (You can't look at it.)"),
        ability(cost("{1}{R}").and(DISCARD_HAND).and(SACRIFICE_THIS), return_exiled, "{1}{R}, Discard your hand, Sacrifice this enchantment: Put all cards exiled with this enchantment into their owners' hands."),
    ],
    ..oracle::CONNECTING_THE_DOTS
};
