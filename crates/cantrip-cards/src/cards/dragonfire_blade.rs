use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::hexproof_from;
use cantrip_oracle as oracle;

/// "Equipped creature gets +2/+2" (layer 7c).
fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

/// "This ability costs {1} less to activate for each color of the creature it targets" (CR 601.2f, 602.2b): the colors
/// the target has as the total cost is determined, once it is announced (CR 601.2c); generic mana only (CR 118.7a), so
/// a creature of five colors takes the {4} to {0} and no lower. A colorless creature takes nothing off.
fn less_per_color_of_target(q: &dyn Query, _: &Asker, _: Costed, targets: &[Target]) -> CostDelta {
    // Its one target, the creature (equip's only slot).
    let colors: u32 = targets.iter().filter_map(|t| t.object()).map(|creature| q.chars(creature).colors.0.count_ones()).sum();
    CostDelta::less(colors as u8)
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{4}"), "Equip {4}. This ability costs {1} less to activate for each color of the creature it targets.")
        .cost_change(less_per_color_of_target)],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_two_plus_two, "Equipped creature gets +2/+2."),
        static_ability(
            Layer::Ability,
            attached_object,
            hexproof_from::gives::<{ HexproofFrom::MONOCOLORED.0 }>,
            "Equipped creature has hexproof from monocolored.",
        ),
    ],
    ..oracle::DRAGONFIRE_BLADE
};
