use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "When you discard this card" (CR 701.9, 113.6k, 603.10): the discarded card in its resulting zone, not another
/// Titanbones, and not a card merely milled or destroyed.
fn discarded(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Discarded { card, .. } if *card == source => Fired::PLAIN,
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOU_GAIN_LIFE, effects::two_plus_one_counters_on_this, "Whenever you gain life, put two +1/+1 counters on Titanbones."),
        triggered(when(EventMask::DISCARDED, discarded), |g, item| effects::gain_life(g, item, 3), "When you discard this card, you gain 3 life.")
            .functions_in(WHERE_A_DISCARD_ENDS),
    ],
    ..oracle::TITANBONES_TOWERING_HEART
};
