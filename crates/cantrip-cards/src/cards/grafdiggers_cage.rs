use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creature cards in graveyards and libraries can't enter the battlefield" (CR 101.2): anyone's, by any effect or rule.
/// The card is judged as it exists in that zone (the card's ruling: a Phyrexian Metamorph there can't enter whatever it
/// would copy), and a card turned face down to be manifested or cloaked is a face-down creature card there (CR 701.40a,
/// 701.40f), so it isn't manifested.
fn no_creature_cards_from_graveyards_or_libraries(q: &dyn Query, _: &Asker, card: ObjectId) -> bool {
    !(matches!(q.obj(card).zone.kind(), ZoneKind::Graveyard | ZoneKind::Library) && q.chars(card).is(CardType::Creature))
}

/// "Players can't cast spells from graveyards or libraries" (CR 101.2, 601.3): anyone's, from any of them. A card exiled
/// from a library and cast is cast from exile, and playing a land is not casting (CR 305.1).
fn no_casting_from_graveyards_or_libraries(_: &dyn Query, _: &Asker, _: PlayerId, from: ZoneKind) -> bool {
    !matches!(from, ZoneKind::Graveyard | ZoneKind::Library)
}

pub static CARD: CardDef = CardDef {
    enter_rules: &[no_creature_cards_from_graveyards_or_libraries],
    cast_rules: &[no_casting_from_graveyards_or_libraries],
    ..oracle::GRAFDIGGERS_CAGE
};
