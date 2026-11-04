use cantrip_api::*;
use cantrip_mechanics::{lands, statics};
use cantrip_oracle as oracle;

/// "Adelbert Steiner gets +1/+1 for each Equipment you control" (CR 613.4c, layer 7c): attached or not, creature or not.
fn plus_one_plus_one_per_equipment_you_control(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let n = lands::count_you_control(view, source, &|c| c.has_subtype(Subtype::Equipment));
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        statics::itself,
        plus_one_plus_one_per_equipment_you_control,
        "Adelbert Steiner gets +1/+1 for each Equipment you control.",
    )],
    ..oracle::ADELBERT_STEINER
};
