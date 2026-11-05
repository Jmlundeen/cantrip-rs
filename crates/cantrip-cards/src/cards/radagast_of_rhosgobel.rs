use cantrip_api::*;
use cantrip_mechanics::cost_reduction;
use cantrip_oracle as oracle;

fn creature_spell(chars: &Chars) -> bool {
    chars.is(CardType::Creature)
}

/// "The first creature spell you cast each turn costs {2} less to cast" (CR 601.2f, 118.7): the ruling is that Radagast need not
/// have been on the battlefield for the spell before, a creature spell cast this turn earlier is the first, and a spell that was not
/// a creature spell is not.
fn first_creature_spell_costs_two_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    cost_reduction::first_spell_you_cast_each_turn(q, source, player, costed, 2, creature_spell)
}

/// "... and can be cast as though it had flash" (CR 307.5, 702.8): the same spell, its controller's, from whatever zone they cast
/// it from.
fn first_creature_spell_you_cast(q: &dyn Query, asker: &Asker, spell: ObjectId, caster: PlayerId) -> bool {
    caster == asker.controller && cost_reduction::first_spell_you_cast_this_turn(q, spell, caster, creature_spell)
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[first_creature_spell_costs_two_less],
    flash_rules: &[first_creature_spell_you_cast],
    ..oracle::RADAGAST_OF_RHOSGOBEL
};
