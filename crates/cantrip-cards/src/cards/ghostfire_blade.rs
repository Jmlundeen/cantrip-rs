use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "Equipped creature gets +2/+2" (layer 7c).
fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

/// "This Equipment's equip ability costs {2} less to activate if it targets a colorless creature" (CR 601.2f, 602.2b):
/// the target as the total cost is determined, once it is announced (CR 601.2c); generic mana only (CR 118.7a). A
/// face-down creature and most artifact creatures are colorless (the ruling).
fn less_for_a_colorless_target(q: &dyn Query, _: ObjectId, _: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    match targets.first() {
        Some(&Target::Object(creature)) if q.chars(creature).colors.is_empty() => CostDelta::less(2),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{3}"), "Equip {3}").cost_change(less_for_a_colorless_target)],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_two_plus_two, "Equipped creature gets +2/+2.")],
    ..oracle::GHOSTFIRE_BLADE
};
