use cantrip_api::*;
use cantrip_mechanics::triggers::CREATURE_SPELL_YOU_CAST;
use cantrip_oracle as oracle;

/// "Black creature spells you cast cost {1} less to cast" (CR 601.2f): asked of the spell's own characteristics as it
/// is cast; only its controller's spells, and a generic reduction takes only the generic part (CR 118.7c).
fn black_creature_spells_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    let chars = q.chars(spell);
    if chars.is(CardType::Creature) && chars.colors.has(q.words(source).color(Color::Black)) { CostDelta::less(1) } else { CostDelta::NONE }
}

/// Each opponent loses 1 life and you gain 1 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[black_creature_spells_cost_less],
    abilities: &[triggered(CREATURE_SPELL_YOU_CAST, drain, "Whenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.")],
    ..oracle::BONTUS_MONUMENT
};
