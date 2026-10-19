use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast if you control a legendary creature" (CR 601.2f): judged once, as the total cost
/// is determined, and generic mana only (CR 118.7a); losing the creature afterwards changes nothing paid.
fn less_with_a_legend(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    let legend = q
        .state()
        .battlefield()
        .iter()
        .any(|&id| q.chars(id).is(CardType::Creature) && q.chars(id).supertypes.legendary && q.controller(id) == player);
    if legend { CostDelta::less(1) } else { CostDelta::NONE }
}

/// "Wrath of the Bloodmane deals 4 damage to target creature or planeswalker." (CR 120.1.)
fn deal_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], deal_four),
    own_cost_changes: &[less_with_a_legend],
    ..oracle::WRATH_OF_THE_BLOODMANE
};
