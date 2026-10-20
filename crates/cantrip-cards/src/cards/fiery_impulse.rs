use cantrip_api::*;
use cantrip_mechanics::{damage_target, spell_mastery, targets};
use cantrip_oracle as oracle;

/// "Fiery Impulse deals 2 damage to target creature. Spell mastery - If there are two or more instant and/or sorcery cards in
/// your graveyard, Fiery Impulse deals 3 damage to that creature instead." (CR 120.3, 614.1 - an ability word, CR 207.2c, checked
/// as this resolves; the spell itself is on the stack, not in the graveyard.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mastery = spell_mastery(g, item.controller);
    damage_target(g, item, 0, if mastery { 3 } else { 2 });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::FIERY_IMPULSE };
