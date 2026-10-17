use cantrip_api::*;
use cantrip_oracle as oracle;

/// "As long as this artifact is untapped, each spell that would cost less than three mana to cast costs three mana to
/// cast": every player's spells, and no ability. Applied once the total cost is otherwise determined - the mana cost or
/// alternative cost, plus increases, less reductions (CR 601.2f) - with generic mana making up the difference, and read
/// as the total is determined: tapping or losing it while the cost is paid changes nothing (its ruling). The spell's mana
/// value is unchanged (CR 202.3).
fn at_least_three(q: &dyn Query, source: ObjectId, _: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(_) if !q.obj(source).tapped => CostDelta::at_least(3),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef { cost_changes: &[at_least_three], ..oracle::TRINISPHERE };
