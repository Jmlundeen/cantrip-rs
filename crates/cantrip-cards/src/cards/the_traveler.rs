use cantrip_api::*;
use cantrip_mechanics::lands::differently_named_lands_in_layers;
use cantrip_oracle as oracle;

/// "Strange new worlds — The Traveler's power is equal to the number of differently named lands you control." (CR 604.3,
/// 208.2a, 201.2a-b, 613.4a): a characteristic-defining ability, so it applies in every zone and counts the largest set
/// of lands its controller has with no name in common.
fn power_is_differently_named_lands(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.power = Some(differently_named_lands_in_layers(q, q.controller(source)) as i32);
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(
        Layer::PtDefine,
        power_is_differently_named_lands,
        "Strange new worlds — The Traveler's power is equal to the number of differently named lands you control.",
    )],
    ..oracle::THE_TRAVELER
};
