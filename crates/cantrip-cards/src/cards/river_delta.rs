use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::depletion_land(Color::Blue, Color::Black, "{T}: Add {U} or {B}. Put a depletion counter on this land."),
    untap_bans: &[lands::depletion_prevents_untap],
    ..oracle::RIVER_DELTA
};
