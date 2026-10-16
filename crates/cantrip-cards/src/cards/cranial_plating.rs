use cantrip_api::*;
use cantrip_mechanics::attach::{attach_to_target, attached_object, equip};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "+1/+0 for each artifact you control" (layer 7c): "you" is the Equipment's controller (CR 109.5), the Equipment
/// itself an artifact among them.
fn plus_power_per_artifact(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let you = state.obj(source).controller;
    let n = state
        .battlefield()
        .iter()
        .filter(|&&id| state.obj(id).controller == you && view.chars(id).is(CardType::Artifact))
        .count() as i16;
    chars.power = chars.power.map(|v| v + n);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{B}{B}"), attach_to_target, "{B}{B}: Attach this Equipment to target creature you control.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        equip(mana("{1}"), "Equip {1}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_power_per_artifact, "Equipped creature gets +1/+0 for each artifact you control.")],
    ..oracle::CRANIAL_PLATING
};
