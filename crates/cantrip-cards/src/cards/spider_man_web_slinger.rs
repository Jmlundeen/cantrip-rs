use cantrip_api::*;
use cantrip_mechanics::web_slinging;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{W}"), "Web-slinging {W}")],
    ..oracle::SPIDER_MAN_WEB_SLINGER
};
