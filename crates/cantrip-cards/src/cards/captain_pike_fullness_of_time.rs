use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "Captain Pike's power and toughness are each equal to the number of differently named lands you control." (CR 604.3,
/// 613.4a): characteristic-defining, so it applies in every zone and reads current land names through the layers. The two
/// permissions use the same game rules as Ka-Zar and Traveling Chocobo.
pub static CARD: CardDef = CardDef {
    play_rules: &[PlayRule::LookAtLibraryTop, PlayRule::LibraryTopLands],
    statics: &[characteristic_defining(
        Layer::PtDefine,
        lands::power_and_toughness_are_differently_named_lands_you_control,
        "Captain Pike's power and toughness are each equal to the number of differently named lands you control.",
    )
    .lost_pt(0, 0)],
    ..oracle::CAPTAIN_PIKE_FULLNESS_OF_TIME
};
