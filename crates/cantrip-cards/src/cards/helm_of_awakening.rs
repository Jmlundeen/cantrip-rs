use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Spells cost {1} less to cast" (CR 601.2f, 118.7a): every player's spells, its controller's included; it takes off
/// generic mana only, and nothing below {0}.
fn discount(_: &dyn Query, _: ObjectId, _: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(_) => CostDelta::less(1),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef { cost_changes: &[discount], ..oracle::HELM_OF_AWAKENING };
