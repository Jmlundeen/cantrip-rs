use cantrip_api::*;
use cantrip_mechanics::{effects, triggers, your_turn};
use cantrip_oracle as oracle;

/// "Whenever one or more cards leave your graveyard during your turn" (CR 603.2c, 603.10a): cards leaving the source's
/// controller's graveyard (`triggers::cards_leave_your_graveyard`), while it is that controller's turn. The cards that left
/// together are the subjects of the matching events, so the batch triggers it once.
fn cards_leave_your_graveyard_during_your_turn(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    your_turn(q, me).then(|| triggers::cards_leave_your_graveyard(q, me, event)).flatten()
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, cards_leave_your_graveyard_during_your_turn),
        effects::plus_one_counter_on_this,
        "Whenever one or more cards leave your graveyard during your turn, put a +1/+1 counter on this creature.",
    )
    .once_per_batch()],
    ..oracle::ATTUNED_HUNTER
};
