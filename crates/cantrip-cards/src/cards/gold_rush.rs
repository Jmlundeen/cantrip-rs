use cantrip_api::*;
use cantrip_mechanics::{pump, targets, tokens};
use cantrip_oracle as oracle;

/// "Create a Treasure token. Until end of turn, up to one target creature gets +2/+2 for each Treasure you control."
/// The Treasure is made first (CR 608.2c, in order), so it counts itself; the Treasures are counted as the spell resolves
/// and the bonus is fixed then (CR 611.2c). With a target chosen that has become illegal the whole spell is countered
/// (CR 608.2b), so no Treasure is made; with no target chosen it still resolves and makes the Treasure.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_a_treasure(g, item)?;
    if let Some(creature) = g.state().spec_targets(item, 0).next().and_then(|slot| targets::object(g, item, slot)) {
        let bonus = 2 * tokens::treasures_controlled(g, item.controller).len() as i16;
        pump(g, item.object, creature, bonus, bonus);
    }
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];

pub static CARD: CardDef = CardDef {
    spell: spell(&[up_to_one(targets::CREATURE.filter).labelled("Creature to get +2/+2 for each Treasure you control until end of turn")], effect),
    ..oracle::GOLD_RUSH
};
