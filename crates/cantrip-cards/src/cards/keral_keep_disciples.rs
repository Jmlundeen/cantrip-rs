use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever you activate a loyalty ability of a Chandra planeswalker" (CR 606.2, 603.2): its controller's activation of a
/// loyalty ability of a permanent that is a Chandra planeswalker (CR 205.3j) as the costs were paid, as it last was if paying
/// them took it away.
fn you_activate_chandra_loyalty_ability(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let chars = q.chars(triggers::loyalty_ability_you_activated(q, me, event)?);
    if chars.is(CardType::Planeswalker) && chars.has_subtype(Subtype::Chandra) { Fired::PLAIN } else { None }
}

/// "This creature deals 1 damage to each opponent": one damage event to each, dealt at once (CR 120.2), by this creature as
/// it last was if it has left (CR 113.7a).
fn ping_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(item.controller).map(|p| (item.object, Target::Player(p), 1)).collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ABILITY_ACTIVATED, you_activate_chandra_loyalty_ability),
        ping_each_opponent,
        "Whenever you activate a loyalty ability of a Chandra planeswalker, this creature deals 1 damage to each opponent.",
    )],
    ..oracle::KERAL_KEEP_DISCIPLES
};
