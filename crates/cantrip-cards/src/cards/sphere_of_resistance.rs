use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Spells cost {1} more to cast" (CR 601.2f): every player's spells, its controller's included; abilities are not spells.
fn tax(_: &dyn Query, _: ObjectId, _: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(_) => CostDelta::more(1),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef { cost_changes: &[tax], ..oracle::SPHERE_OF_RESISTANCE };
