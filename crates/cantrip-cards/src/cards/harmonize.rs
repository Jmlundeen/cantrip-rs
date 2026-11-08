use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Draw three cards." (CR 121.1): three cards to the spell's controller and nobody else - the shared `effects::draw`, which
/// Bankrupt in Blood and Lórien Revealed print too - and a library asked for more than it has is a loss (CR 704.5b).
pub static CARD: CardDef = CardDef {
    spell: spell(&[], |g, item| cantrip_mechanics::effects::draw(g, item, 3)),
    ..oracle::HARMONIZE
};
