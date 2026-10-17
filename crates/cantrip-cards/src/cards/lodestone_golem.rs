use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Nonartifact spells cost {1} more to cast" (CR 601.2f): every player's, read as the spell is cast (an artifact
/// creature spell is an artifact spell, and is not taxed).
fn tax(q: &dyn Query, _: ObjectId, _: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(id) if !q.chars(id).is(CardType::Artifact) => CostDelta::more(1),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef { cost_changes: &[tax], ..oracle::LODESTONE_GOLEM };
