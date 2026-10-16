use cantrip_api::*;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(lands::enters_tapped_unless_you_control_basic_land),
    abilities: &[
        mana_ability(TAP, add(Mana::U), "{T}: Add {U}."),
        ability(cost("{2}{U}, {T}"), effects::loot, "{2}{U}, {T}: Draw a card, then discard a card."),
    ],
    ..oracle::AGNA_QELA
};
