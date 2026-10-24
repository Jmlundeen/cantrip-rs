use cantrip_api::*;
use cantrip_mechanics::statics::players_cant_cast_from_graveyards_or_libraries;
use cantrip_oracle as oracle;

/// "Creature cards in graveyards and libraries can't enter the battlefield" (CR 101.2): anyone's, by any effect or rule.
/// The card is judged as it exists in that zone (the card's ruling: a Phyrexian Metamorph there can't enter whatever it
/// would copy), and a card turned face down to be manifested or cloaked is a face-down creature card there (CR 701.40a,
/// 701.40f), so it isn't manifested.
fn no_creature_cards_from_graveyards_or_libraries(q: &dyn Query, _: &Asker, card: ObjectId) -> bool {
    !(matches!(q.obj(card).zone.kind(), ZoneKind::Graveyard | ZoneKind::Library) && q.chars(card).is(CardType::Creature))
}

pub static CARD: CardDef = CardDef {
    enter_rules: &[no_creature_cards_from_graveyards_or_libraries],
    cast_rules: &[players_cant_cast_from_graveyards_or_libraries],
    ..oracle::GRAFDIGGERS_CAGE
};
