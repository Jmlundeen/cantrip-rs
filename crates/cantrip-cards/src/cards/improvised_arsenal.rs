use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::copy::create_token_copy_of_it;
use cantrip_mechanics::statics::artifacts_you_control;
use cantrip_oracle as oracle;

/// "Equipped creature gets +1/+0 for each artifact you control" (CR 613.4c, layer 7c): "you" is the Equipment's controller,
/// and the Equipment itself counts as an artifact (a token copy of it too). The count is made as the layers are computed, so it
/// follows every artifact that enters or leaves.
fn plus_one_zero_per_artifact(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let n = artifacts_you_control(view, source);
    chars.power = chars.power.map(|v| v + n);
}

/// "{4}{R}: Create a token that's a copy of this Equipment" (CR 707.2, 111.4): its copiable values, or its last known ones if
/// it has left the battlefield by now (CR 608.2h). The token is created unattached.
fn copy_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_token_copy_of_it(g, item, |_, values| values)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{4}{R}"), copy_this, "{4}{R}: Create a token that's a copy of this Equipment."),
        equip(mana("{R}"), "Equip {R}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_one_zero_per_artifact, "Equipped creature gets +1/+0 for each artifact you control.")],
    ..oracle::IMPROVISED_ARSENAL
};
