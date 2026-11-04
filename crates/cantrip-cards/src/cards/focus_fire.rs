use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Focus Fire deals X damage to target attacking or blocking creature, where X is 2 plus the number of creatures and/or
/// Spacecraft you control." X is counted as this resolves (CR 608.2h); a Spacecraft that is also a creature counts once,
/// and the subtype is read through this spell's words (CR 612.2). If the target is no longer legal nothing happens (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spacecraft = g.words(item.object).subtype(Subtype::Spacecraft);
    let counted = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Creature) || chars.has_subtype(spacecraft)
        })
        .count();
    damage_target(g, item, 0, 2 + counted as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ATTACKING_OR_BLOCKING_CREATURE], effect),
    ..oracle::FOCUS_FIRE
};
