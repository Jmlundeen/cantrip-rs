use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +3/-3 and loses all creature types until end of turn." Both are continuous effects until the
/// cleanup step (CR 514.2): the creature types go in layer 4 (CR 613.1d), Changeling's among them, and a creature type
/// given later is had (the rulings); +3/-3 applies in layer 7c (CR 613.4c), so a 3-toughness creature dies of it
/// (CR 704.5f). A target that is no longer legal is not changed (CR 608.2b), which with its one target means the spell
/// does not resolve. Changeling makes the card itself every creature type in every zone (CR 702.73a).
fn invert(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 3, -3);
        g.add_effect(item.object, creature, EffectKind::LoseAllCreatureTypes, Duration::EndOfTurn);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], invert), ..oracle::NAMELESS_INVERSION };
