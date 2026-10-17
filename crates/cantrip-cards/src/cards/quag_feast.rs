use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature, planeswalker, or Vehicle" (CR 115.1, 301.7): a permanent on the battlefield that is a creature, a
/// planeswalker or a Vehicle (an artifact type, which is a target whether or not it is a creature right now).
fn creature_planeswalker_or_vehicle(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) || c.is(CardType::Planeswalker) || c.has_subtype(Subtype::Vehicle)
    })
}

/// "Mill two cards, then destroy the chosen permanent if its mana value is less than or equal to the number of cards in your
/// graveyard." (CR 701.17, 701.8, 202.3.) The milled cards count, the spell, still on the stack, does not; the mana value is
/// the permanent's as it stands once the cards are milled. A target that is gone or illegal makes the whole spell do nothing
/// (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    g.mill(item.controller, 2);
    let in_graveyard = g.state().zone(Zone::graveyard(item.controller)).len() as u32;
    if g.mana_value(permanent) <= in_graveyard {
        g.destroy(permanent);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(creature_planeswalker_or_vehicle)], effect),
    ..oracle::QUAG_FEAST
};
