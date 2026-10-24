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

/// Every zone a discarded card can wind up in, a replacement effect's included (CR 113.6k, 701.9, 614.1), but the library:
/// the engine never looks in a library for a triggered ability (`CardTable::functions_elsewhere`), so naming it would
/// refuse the whole card table. A core gap: a discard replaced to put the card into a library is not yet seen.
const WHERE_A_DISCARD_ENDS: ZoneKinds =
    ZoneKinds::of(&[ZoneKind::Graveyard, ZoneKind::Exile, ZoneKind::Hand, ZoneKind::Battlefield]);

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOU_GAIN_LIFE, effects::two_plus_one_counters_on_this, "Whenever you gain life, put two +1/+1 counters on Titanbones."),
        triggered(when(EventMask::DISCARDED, discarded), |g, item| effects::gain_life(g, item, 3), "When you discard this card, you gain 3 life.")
        .functions_in(WHERE_A_DISCARD_ENDS),
    ],
    ..oracle::TITANBONES_TOWERING_HEART
};
