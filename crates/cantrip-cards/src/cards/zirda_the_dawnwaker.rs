use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Abilities you activate that aren't mana abilities cost {2} less to activate. This effect can't reduce the mana in
/// that cost to less than one mana" (CR 601.2f, 118.7): the core asks only non-mana abilities, whatever the zone they
/// are activated from. Mana abilities are not priced this way (they are paid as the cost they are part of is), and a special
/// action is no ability (CR 116.1): `Costed::Ability` alone.
fn two_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    if matches!(costed, Costed::Ability(..)) && player == q.controller(source) { CostDelta::less_to_one(2) } else { CostDelta::NONE }
}

/// {1}, {T}: Target creature can't block this turn.
fn cant_block(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::CantBlock, Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}, {T}"), cant_block, "{1}, {T}: Target creature can't block this turn.")
        .targets(&[targets::CREATURE])],
    cost_changes: &[two_less],
    ..oracle::ZIRDA_THE_DAWNWAKER
};
