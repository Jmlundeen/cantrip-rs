use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target blocking creature you control" (CR 115.1, 509.1): a creature controlled by the caster that is blocking now.
fn blocking_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.chars(id).is(CardType::Creature)
        && q.controller(id) == asker.controller
        && q.state().blocks.iter().any(|b| b.blocker == id))
}

/// "+2/+2 until end of turn" then "Draw a card" (CR 611.2a, 121.1): two instructions in order; a target that is no longer
/// a legal blocking creature means nothing happens at all (CR 608.2b), so the card is not drawn either.
fn pump_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::pump(g, item.object, creature, 2, 2);
    g.end_sentence();
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(blocking_creature_you_control)], pump_and_draw),
    ..oracle::EVASIVE_MANEUVERS
};
