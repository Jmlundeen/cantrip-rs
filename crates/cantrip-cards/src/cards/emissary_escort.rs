use cantrip_api::*;
use cantrip_mechanics::statics::itself;
use cantrip_oracle as oracle;

/// "This creature gets +X/+0, where X is the greatest mana value among other artifacts you control" (CR 613.4c, layer 7c): the
/// artifacts under its controller's control as the earlier layers made them, the Escort itself not among them (it is an artifact
/// too), each judged on its mana value as the layers have it (CR 202.3; {X} in a cost counts as 0, CR 107.3). It applies only on
/// the battlefield, so its power is 0 anywhere else.
fn plus_greatest_mana_value_among_other_artifacts(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let you = state.obj(source).controller;
    let greatest = state
        .battlefield()
        .iter()
        .filter(|&&id| id != source && state.obj(id).controller == you)
        .filter_map(|&id| {
            let c = view.chars(id);
            c.is(CardType::Artifact).then(|| view.mana_value(id, &c))
        })
        .max()
        .unwrap_or(0) as i32;
    chars.power = chars.power.map(|v| v + greatest);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        itself,
        plus_greatest_mana_value_among_other_artifacts,
        "This creature gets +X/+0, where X is the greatest mana value among other artifacts you control.",
    )],
    ..oracle::EMISSARY_ESCORT
};
