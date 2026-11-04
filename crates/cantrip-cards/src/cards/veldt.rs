use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &lands::depletion_land(Color::Green, Color::White, "{T}: Add {G} or {W}. Put a depletion counter on this land."),
    untap_bans: &[lands::depletion_prevents_untap],
    ..oracle::VELDT
};
