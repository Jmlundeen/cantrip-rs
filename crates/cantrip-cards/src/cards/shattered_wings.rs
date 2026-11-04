use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Destroy target artifact, enchantment, or creature with flying. Surveil 1." (CR 701.8a, 701.25): the surveil is the second
/// sentence; with its only target illegal the spell does not resolve, so there is none (CR 608.2b).
fn destroy_then_surveil(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    g.destroy(target);
    effects::surveil(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_ENCHANTMENT_OR_CREATURE_WITH_FLYING], destroy_then_surveil),
    ..oracle::SHATTERED_WINGS
};
