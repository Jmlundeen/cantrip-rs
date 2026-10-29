use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "Captain Pike's power and toughness are each equal to the number of differently named lands you control." (CR 604.3,
/// 613.4a): characteristic-defining, so it applies in every zone and reads current land names through the layers.
fn size(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let controller = q.controller(source);
    let count = lands::differently_named_lands_in_layers(q, controller) as i16;
    chars.power = Some(count);
    chars.toughness = Some(count);
}

/// The two permissions use the same game rules as Ka-Zar and Traveling Chocobo.
pub static CARD: CardDef = CardDef {
    play_rules: &[PlayRule::LookAtLibraryTop, PlayRule::LibraryTopLands],
    statics: &[characteristic_defining(
        Layer::PtDefine,
        size,
        "Captain Pike's power and toughness are each equal to the number of differently named lands you control.",
    )
    .lost_pt(0, 0)],
    ..oracle::CAPTAIN_PIKE_FULLNESS_OF_TIME
};
