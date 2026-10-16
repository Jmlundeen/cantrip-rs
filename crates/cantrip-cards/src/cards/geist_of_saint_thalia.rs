use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Noncreature spells you cast cost {1} less to cast" (CR 601.2f): only its controller's spells, judged on the spell's
/// own characteristics as it is cast; a generic reduction takes only the generic part (CR 118.7c). Lands are never cast
/// and abilities are not spells, so neither is asked.
fn noncreature_spells_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    if q.chars(spell).is(CardType::Creature) { CostDelta::NONE } else { CostDelta::less(1) }
}

pub static CARD: CardDef = CardDef { cost_changes: &[noncreature_spells_cost_less], ..oracle::GEIST_OF_SAINT_THALIA };
