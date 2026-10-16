use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

pub static EXTRA: &[&CardDef] = &[&tokens::PEST];

/// Destroy up to X target artifacts and/or enchantments. Create twice X Pest tokens, all at once (CR 101.4). The tokens do
/// not depend on any target still being legal: only a spell with no legal target at all does nothing (CR 608.2b), which the
/// engine settles before the effect runs.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.destroy_all(&targets::legal_objects(g, item));
    tokens::create(g, item.controller, &tokens::PEST, item.x as u32 * 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_ENCHANTMENT.between(0, TargetSpec::COUNTED)], effect),
    ..oracle::PEST_INFESTATION
};
