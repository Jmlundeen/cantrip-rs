use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "This spell costs {1} more to cast for each target beyond the first" (CR 601.2f): read once the targets are announced
/// (CR 601.2c), so zero targets or one costs nothing more. Its mana value is its mana cost's alone (CR 202.3).
fn more_per_extra_target(_: &dyn Query, _: ObjectId, _: PlayerId, costed: Costed, targets: &[Target]) -> CostDelta {
    let chosen = targets.iter().filter(|t| **t != Target::None).count();
    match costed {
        Costed::Spell(_) if chosen > 1 => CostDelta::more((chosen - 1).min(255) as u8),
        _ => CostDelta::NONE,
    }
}

/// "Fireball deals X damage divided evenly, rounded down, among any number of targets": divided as it resolves, among the
/// targets still legal then (CR 608.2b; its rulings), each dealt X divided by how many those are, rounded down, all at once
/// (CR 120.3). With no legal target it deals none.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let legal: Vec<Target> = (0..item.target_count as usize).filter(|&slot| g.target_legal(item, slot)).map(|slot| item.targets[slot]).collect();
    if legal.is_empty() {
        return Ok(());
    }
    let each = item.x / legal.len() as u16;
    if each > 0 {
        let hits: Vec<(ObjectId, Target, u16)> = legal.into_iter().map(|t| (item.object, t, each)).collect();
        g.damage_together(&hits);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY.between(0, TargetSpec::ANY_NUMBER)], effect),
    own_cost_changes: &[more_per_extra_target],
    ..oracle::FIREBALL
};
