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
        AbilityDef::Activated(ActivatedDef {
            text: "{B}{B}: Attach this Equipment to target creature you control.",
            cost: Cost::mana(mana(0, 0, 0, 2, 0, 0)),
            targets: &[targets::CREATURE_YOU_CONTROL],
            effect: attach_to_target,
            ..ActivatedDef::BLANK
        }),
        equip(mana(1, 0, 0, 0, 0, 0), "Equip {1}"),
    ],
    statics: &[StaticDef {
        text: "Equipped creature gets +1/+0 for each artifact you control.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: plus_power_per_artifact,
        ..StaticDef::BLANK
    }],
    ..oracle::CRANIAL_PLATING
};
