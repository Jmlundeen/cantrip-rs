use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target artifact or enchantment. If that permanent was a legendary enchantment, draw a card." Read
/// the target’s characteristics before destroying it (CR 608.2h).
fn destroy_and_maybe_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let was_legendary_enchantment = targets::object(g, item, 0).is_some_and(|id| {
        let chars = g.chars(id);
        chars.is(CardType::Enchantment) && chars.supertypes.legendary
    });
    destroy_target(g, item, 0);
    if was_legendary_enchantment {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(SACRIFICE_THIS, destroy_and_maybe_draw, "Sacrifice this creature: Destroy target artifact or enchantment. If that permanent was a legendary enchantment, draw a card. Activate only as a sorcery.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT])
        .sorcery_speed()],
    ..oracle::BUDDING_INSURGENT
};
