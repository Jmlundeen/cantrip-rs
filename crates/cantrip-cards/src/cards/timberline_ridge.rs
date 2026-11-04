use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::depletion_land(Color::Red, Color::Green, "{T}: Add {R} or {G}. Put a depletion counter on this land."),
    untap_bans: &[lands::depletion_prevents_untap],
    ..oracle::TIMBERLINE_RIDGE
};
