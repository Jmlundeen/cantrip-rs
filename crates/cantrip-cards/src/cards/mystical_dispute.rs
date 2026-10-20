use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// "This spell costs {2} less to cast if it targets a blue spell" (CR 601.2c, 601.2f): the target as the total cost is
/// determined, once it is announced; generic mana only (CR 118.7a). "Blue" is read through the words of this spell's own
/// text (CR 612). The target's color is its color on the stack, as the cost is determined.
fn less_for_a_blue_spell(q: &dyn Query, source: ObjectId, _: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let blue = q.words(source).color(Color::Blue);
    if targets.iter().filter_map(|t| t.object()).any(|spell| q.chars(spell).colors.has(blue)) {
        CostDelta::less(2)
    } else {
        CostDelta::NONE
    }
}

/// "Counter target spell unless its controller pays {3}" (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{3}"))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    own_cost_changes: &[less_for_a_blue_spell],
    ..oracle::MYSTICAL_DISPUTE
};
