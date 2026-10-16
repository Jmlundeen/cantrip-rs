use cantrip_api::*;
use cantrip_oracle as oracle;

/// Noncreature spells cost {1} more to cast (CR 601.2f), for every player. Abilities and lands (never cast) are not asked.
fn tax(q: &dyn Query, _source: ObjectId, _player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(id) if !q.chars(id).is(CardType::Creature) => CostDelta::more(1),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef { cost_changes: &[tax], ..oracle::THALIA_GUARDIAN_OF_THRABEN };
