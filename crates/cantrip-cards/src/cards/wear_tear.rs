use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// Wear: "Destroy target artifact."
fn wear(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// Tear: "Destroy target enchantment."
fn tear(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static WEAR: CardDef = CardDef { spell: spell(&[targets::ARTIFACT], wear), ..oracle::WEAR };

pub static TEAR: CardDef = CardDef { spell: spell(&[targets::ENCHANTMENT], tear), ..oracle::TEAR };

/// "Fuse (You may cast one or both halves of this card from your hand.)" (CR 702.102): the core makes the fused split
/// spell from the halves.
pub static CARD: CardDef = CardDef { split: Some((&WEAR, &TEAR)), fuse: true, ..oracle::WEAR_TEAR };

pub static EXTRA: &[&CardDef] = &[&WEAR, &TEAR];
