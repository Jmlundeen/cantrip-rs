use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::depletion_land(Color::White, Color::Blue, "{T}: Add {W} or {U}. Put a depletion counter on this land."),
    untap_bans: &[lands::depletion_prevents_untap],
    ..oracle::LAND_CAP
};
