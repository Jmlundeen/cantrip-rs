use cantrip_api::*;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Destroy target artifact or enchantment. If its mana value was 2 or less, create a Lander token." (CR 701.8, 202.3.) The
/// mana value is the permanent's as it stands as the spell resolves, read before it is destroyed so that a copy or a token
/// copy has its copied value (CR 707.2) whether or not the destroy happens: indestructible or regenerated, it was still a
/// mana value of 2 or less, and the Lander is made all the same. A target that is gone does nothing (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let value = g.mana_value(permanent);
    g.destroy(permanent);
    if value <= 2 {
        tokens::create_lander(g, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_ENCHANTMENT], effect),
    ..oracle::SEEDSHIP_IMPACT
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
