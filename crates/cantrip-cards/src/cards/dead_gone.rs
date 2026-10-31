use cantrip_api::*;
use cantrip_mechanics::{damage_target, effects, targets};
use cantrip_oracle as oracle;

/// Dead: "Dead deals 2 damage to target creature."
fn dead(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static DEAD: CardDef = CardDef { spell: spell(&[targets::CREATURE], dead), ..oracle::DEAD };

/// Gone: "Return target creature you don't control to its owner's hand."
pub static GONE: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_DONT_CONTROL], effects::return_first_target_to_hand),
    ..oracle::GONE
};

pub static CARD: CardDef = CardDef { split: Some((&DEAD, &GONE)), ..oracle::DEAD_GONE };

pub static EXTRA: &[&CardDef] = &[&DEAD, &GONE];
