use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever a creature an opponent controls with the greatest power among creatures that player controls dies" (CR 700.4, 603.10a,
/// 208.1): the creature as it last was, under the controller it last had, an opponent of this ability's controller, with no creature
/// that player controlled just before having more power then, counting those that died with it. A look-back ability is asked of the
/// game as it was immediately before the event, so a creature that a lord dying with the event had pumped still has the bonus. Creatures tied for the greatest power each qualify, so each triggers it.
fn greatest_power_creature_an_opponent_controls_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { old, from, to, .. } = event else { return None };
    if !from.is(ZoneKind::Battlefield) || !to.is(ZoneKind::Graveyard) || !q.chars(*old).is(CardType::Creature) { return None }
    let player = q.controller(*old);
    if !q.state().opponents(me.controller).any(|p| p == player) { return None }
    let power = q.chars(*old).power?;
    let beaten = q.permanents_of(player).into_iter().map(|id| q.chars(id)).any(|c| c.is(CardType::Creature) && c.power.is_some_and(|p| p > power));
    if beaten { None } else { Fired::PLAIN }
}

/// "Draw a card and put a +1/+1 counter on Kraven the Hunter" (CR 121.1, 122.1).
fn draw_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    effects::plus_one_counter_on_this(g, item)
}

/// Trample is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, greatest_power_creature_an_opponent_controls_dies),
        draw_and_grow,
        "Whenever a creature an opponent controls with the greatest power among creatures that player controls dies, draw a card and put a +1/+1 counter on Kraven the Hunter.",
    )],
    ..oracle::KRAVEN_THE_HUNTER
};
