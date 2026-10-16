use cantrip_api::*;
use cantrip_mechanics::{gift, targets, tokens};
use cantrip_oracle as oracle;

/// "Blooming Blast deals 2 damage to target creature. If the gift was promised, Blooming Blast also deals 3 damage to that
/// creature's controller." The controller is the creature's as the spell resolves, before the damage can kill it; a spell
/// whose target has become illegal does nothing (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(creature);
    g.damage(item.object, Target::Object(creature), 2);
    if item.gift.is_some() {
        g.damage(item.object, Target::Player(controller), 3);
    }
    Ok(())
}

/// "Gift a Treasure" (CR 702.174h): the Treasure is a token of its own.
pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    gift: Some(gift::TREASURE),
    ..oracle::BLOOMING_BLAST
};
