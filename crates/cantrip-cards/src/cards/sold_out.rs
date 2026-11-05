use cantrip_api::*;
use cantrip_mechanics::{history, targets, tokens};
use cantrip_oracle as oracle;

/// "Exile target creature. If it was dealt damage this turn, create a Clue token." (CR 701.13a, 701.7a): nothing at all if the creature is no
/// longer a legal target (CR 608.2b). The Clue is created whoever controlled the creature, and creating it is not investigating (the ruling).
fn exile_it_and_create_a_clue_if_it_was_dealt_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let was_dealt_damage = history::was_dealt_damage_this_turn(g.history(), creature);
    g.exile(creature);
    if was_dealt_damage {
        tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], exile_it_and_create_a_clue_if_it_was_dealt_damage),
    ..oracle::SOLD_OUT
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
