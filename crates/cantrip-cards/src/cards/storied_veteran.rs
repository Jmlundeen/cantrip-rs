use cantrip_api::*;
use cantrip_mechanics::cast_from_your_graveyard;
use cantrip_oracle as oracle;

/// "You may cast this card from your graveyard. If you do, it enters with a finality counter on it" (CR 601.3, 614.1c): the
/// card's own permission and its entry rider, which a cast by another permission (Lurrus of the Dream-Den) does not get.
pub static CARD: CardDef = CardDef {
    enters: Some(cast_from_your_graveyard::enters_with_finality_if_cast_by_it),
    exile_casts: &[cast_from_your_graveyard::PERMISSION],
    ..oracle::STORIED_VETERAN
};
