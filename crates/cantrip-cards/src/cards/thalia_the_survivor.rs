use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Noncreature spells your opponents cast cost {1} more to cast" Apply the generic tax when determining an
/// opponent’s spell cost.
fn tax_opponents(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(id) if player != q.controller(source) && !q.chars(id).is(CardType::Creature) => CostDelta::more(1),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[tax_opponents],
    ..oracle::THALIA_THE_SURVIVOR
};
