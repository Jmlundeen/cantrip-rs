use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::unearth;
use cantrip_oracle as oracle;

/// "When this creature enters from your graveyard": its controller's own graveyard, not anyone else's.
fn enters_from_your_graveyard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, from, to, .. }
            if *new == source && to.is(ZoneKind::Battlefield) && *from == Zone::graveyard(q.controller(source)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// You may discard your hand. If you do, draw three cards (an empty hand discarded still counts).
fn rummage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !effects::ask_may(g, player, item.object, Purpose::Optional)? {
        return Ok(());
    }
    g.discard_hand(player);
    g.draw(player, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        unearth::ability(mana(3, 0, 0, 0, 2, 0), "Unearth {3}{R}{R}"),
        triggered(when(EventMask::ZONE_CHANGE, enters_from_your_graveyard), rummage, "When this creature enters from your graveyard, you may discard your hand. If you do, draw three cards."),
    ],
    delayed: &[unearth::EXILE],
    ..oracle::PHYREXIAN_DRAGON_ENGINE
};
