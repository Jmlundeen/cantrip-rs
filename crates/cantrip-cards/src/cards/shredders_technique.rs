use cantrip_api::*;
use cantrip_mechanics::{sneak, targets};
use cantrip_oracle as oracle;

/// "Destroy target creature or enchantment. If an enchantment was destroyed this way, you lose 2 life." The target is read
/// as it is before it is destroyed (an enchantment creature is an enchantment), and the life is lost only if it was
/// destroyed: an indestructible permanent, or one a regeneration shield saved, is not (CR 701.8a, 702.12b, 701.19a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let was_enchantment = g.chars(target).is(CardType::Enchantment);
    let destroyed = g.destroy_all(&[target]);
    if was_enchantment && !destroyed.is_empty() {
        g.lose_life(item.controller, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(targets::creature_or_enchantment)], effect),
    ways: &[sneak::way(mana("{B}"), "Sneak {B}")],
    ..oracle::SHREDDERS_TECHNIQUE
};
