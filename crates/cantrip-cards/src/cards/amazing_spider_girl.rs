use cantrip_api::*;
use cantrip_mechanics::web_slinging;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{2}{W}"), "Web-slinging {2}{W}")],
    ..oracle::AMAZING_SPIDER_GIRL
};
