use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "If it was dealt damage this turn" (CR 120.2, 608.2h): the target was the recipient of damage this turn, combat or
/// not, read from the history before it leaves. Damage that was prevented is no damage event (CR 615.6), and the target is the object that was
/// dealt it, so a creature that left and came back has been dealt none (CR 400.7).
fn was_dealt_damage_this_turn(g: &dyn Game, creature: ObjectId) -> bool {
    g.history().damage_dealt().any(|d| d.target == Target::Object(creature))
}

/// "Exile target creature. If it was dealt damage this turn, create a Clue token." (CR 701.13a, 701.7a): nothing at all if the creature is no
/// longer a legal target (CR 608.2b). The Clue is created whoever controlled the creature, and creating it is not investigating (the ruling).
fn exile_it_and_create_a_clue_if_it_was_dealt_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let was_dealt_damage = was_dealt_damage_this_turn(g, creature);
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
