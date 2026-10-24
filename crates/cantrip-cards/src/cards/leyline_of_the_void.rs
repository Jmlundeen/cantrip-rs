use cantrip_api::*;
use cantrip_mechanics::{replacements, statics};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    opening_hand: Some(statics::BEGIN_ON_THE_BATTLEFIELD),
    // "If a card would be put into an opponent's graveyard from anywhere, exile it instead" (CR 614.1a, 400.3).
    replacements: &[replacement(
        ReplacementKind::ZoneChange,
        replacements::card_to_an_opponents_graveyard,
        replacements::exile_instead_apply,
        "exiles a card an opponent's graveyard would receive",
    )],
    ..oracle::LEYLINE_OF_THE_VOID
};
