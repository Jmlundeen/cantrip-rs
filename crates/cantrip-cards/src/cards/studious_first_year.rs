use cantrip_api::*;
use cantrip_oracle as oracle;

/// Enters prepared (CR 722): its prepare spell is Rampant Growth, which is a card of its own.
pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&super::rampant_growth::CARD),
    ..oracle::STUDIOUS_FIRST_YEAR
};
