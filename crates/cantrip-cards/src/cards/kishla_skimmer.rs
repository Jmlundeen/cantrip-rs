use cantrip_api::*;
use cantrip_mechanics::{effects, triggers, your_turn};
use cantrip_oracle as oracle;

/// "Whenever a card leaves your graveyard during your turn" (CR 603.2, 603.10a): cards leaving the source's controller's graveyard
/// (`triggers::cards_leave_your_graveyard`), while it is that controller's turn.
fn cards_leave_your_graveyard_during_your_turn(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    your_turn(q, me).then(|| triggers::cards_leave_your_graveyard(q, me, event)).flatten()
}

/// "Whenever a card leaves your graveyard during your turn, draw a card. This ability triggers only once each turn."
/// Flying is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, cards_leave_your_graveyard_during_your_turn),
        |g, item| effects::draw(g, item, 1),
        "Whenever a card leaves your graveyard during your turn, draw a card. This ability triggers only once each turn.",
    )
    .triggers_each_turn(1)],
    ..oracle::KISHLA_SKIMMER
};
