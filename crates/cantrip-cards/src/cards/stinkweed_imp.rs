use cantrip_api::*;
use cantrip_mechanics::dredge::dredge;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_A_CREATURE;
use cantrip_oracle as oracle;

/// "... destroy that creature": the creature it dealt combat damage to, the trigger's subject; nothing if it has left the
/// battlefield since (CR 400.7). It is a triggered ability, not deathtouch (its ruling): only combat damage triggers it.
fn destroy_that_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject
        && targets::permanent(g, creature)
    {
        g.destroy(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(DEALS_COMBAT_DAMAGE_TO_A_CREATURE, destroy_that_creature, "Whenever this creature deals combat damage to a creature, destroy that creature.")],
    // "Dredge 5 (If you would draw a card, you may mill five cards instead. If you do, return this card from your graveyard
    // to your hand.)"
    replacements: &[dredge::<5>()],
    ..oracle::STINKWEED_IMP
};
