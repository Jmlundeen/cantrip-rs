use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Return target creature card from your graveyard to the battlefield. The ability exists independently of its source
/// once it is on the stack (CR 113.7a), so the enchantment being in its owner's hand changes nothing; the target was
/// chosen before the creature was sacrificed, so that creature is never a legal one.
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(RETURN_THIS.and(sacrifice(1, filters::creature)), reanimate, "Sacrifice a creature, Return this enchantment to its owner's hand: Return target creature card from your graveyard to the battlefield. Activate only as a sorcery.")
        .sorcery_speed()
        .targets(&[cantrip_mechanics::targets::CREATURE_CARD_IN_YOUR_GRAVEYARD])],
    ..oracle::RECURRING_NIGHTMARE
};
