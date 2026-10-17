use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target creature or enchantment. You lose 2 life." The life is lost whether or not the permanent was destroyed
/// (indestructible, regenerated): the spell is still resolving with a legal target. A spell whose only target is illegal is not
/// resolved at all (CR 608.2b), so no life is lost then.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    g.lose_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(targets::creature_or_enchantment)], effect),
    ..oracle::WITHERING_TORMENT
};
