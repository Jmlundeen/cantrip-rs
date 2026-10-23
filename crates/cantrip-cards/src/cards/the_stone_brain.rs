use cantrip_api::*;
use cantrip_mechanics::library::exile_up_to_four_named_and_draw_for_each_from_hand;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{2}, {T}, Exile The Stone Brain: Choose a card name. Search target opponent's graveyard, hand, and library for up to four
/// cards with that name and exile them. That player shuffles, then draws a card for each card exiled from their hand this way.
/// Activate only as a sorcery." The target is chosen as the ability is put on the stack, the name as it resolves (CR 608.2c).
pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}, {T}").and(EXILE_THIS), exile_up_to_four_named_and_draw_for_each_from_hand, "{2}, {T}, Exile The Stone Brain: Choose a card name. Search target opponent's graveyard, hand, and library for up to four cards with that name and exile them. That player shuffles, then draws a card for each card exiled from their hand this way. Activate only as a sorcery.")
        .targets(&[targets::OPPONENT])
        .sorcery_speed()],
    ..oracle::THE_STONE_BRAIN
};
