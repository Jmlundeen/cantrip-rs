use cantrip_api::*;
use cantrip_mechanics::{effects, flurry};
use cantrip_oracle as oracle;

/// "Whenever you cast your second spell each turn, put a +1/+1 counter on this creature" (CR 603.2): the flurry trigger (flurry
/// being an ability word, CR 207.2c). It triggers only for a spell cast while the Operative is on the battlefield, so the
/// Operative cast as the second spell does not trigger itself, and it resolves before the spell that caused it.
pub static CARD: CardDef = CardDef {
    abilities: &[flurry::trigger("Whenever you cast your second spell each turn, put a +1/+1 counter on this creature.", effects::plus_one_counter_on_this)],
    ..oracle::ILLVOI_OPERATIVE
};
