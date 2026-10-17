use cantrip_api::*;
use cantrip_mechanics::delayed::{self, SACRIFICE_CREATED};
use cantrip_mechanics::statics::{have_haste, your_creatures};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Red spells you cast cost {1} less to cast" (CR 601.2f): asked of the spell's own characteristics as it is cast; only its
/// controller's spells, and a generic reduction takes only the generic part (CR 118.7c). "Red" is a word of the Crystal's text
/// (CR 612).
fn red_spells_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    if q.chars(spell).colors.has(q.words(source).color(Color::Red)) { CostDelta::less(1) } else { CostDelta::NONE }
}

/// "Create a token that's a copy of target creature you control. Sacrifice it at the beginning of the next end step" (CR 707.2,
/// 111.1, 603.7): the delayed ability is about exactly the token this resolution made (CR 603.7c). Nothing happens if the target
/// is no longer legal (CR 608.2b).
fn copy_and_sacrifice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let tokens = g.create_token_copies(creature, item.controller, 1)?;
    delayed::at_next_end_step_about(g, item, 0, &tokens);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[red_spells_cost_less],
    statics: &[static_ability(Layer::Ability, your_creatures, have_haste, "Creatures you control have haste.")],
    abilities: &[ability(cost("{4}{R}{R}, {T}"), copy_and_sacrifice, "{4}{R}{R}, {T}: Create a token that's a copy of target creature you control. Sacrifice it at the beginning of the next end step.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    delayed: &[SACRIFICE_CREATED],
    ..oracle::THE_FIRE_CRYSTAL
};
