use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// Turn: "Until end of turn, target creature loses all abilities and becomes a red Weird with base power and toughness
/// 0/1."
fn turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::loses_all_abilities_and_becomes(g, item, 0, Color::Red, Subtype::Weird, 0, 1);
    Ok(())
}

pub static TURN: CardDef = CardDef { spell: spell(&[targets::CREATURE], turn), ..oracle::TURN };

pub static BURN: CardDef = CardDef { spell: spell(&[targets::ANY], effects::damage_first_target::<2>), ..oracle::BURN };

/// "Fuse (You may cast one or both halves of this card from your hand.)" (CR 702.102): the core makes the fused split
/// spell from the halves.
pub static CARD: CardDef = CardDef { split: Some((&TURN, &BURN)), fuse: true, ..oracle::TURN_BURN };

pub static EXTRA: &[&CardDef] = &[&TURN, &BURN];
