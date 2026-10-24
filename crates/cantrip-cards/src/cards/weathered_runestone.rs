use cantrip_api::*;
use cantrip_mechanics::statics::players_cant_cast_from_graveyards_or_libraries;
use cantrip_mechanics::targets::is_nonland_permanent_card;
use cantrip_oracle as oracle;

/// "Nonland permanent cards in graveyards and libraries can't enter the battlefield" (CR 101.2, 110.4a): anyone's, by any
/// effect or rule. The card is judged as it exists in that zone (the card's rulings: a double-faced card by its front face
/// only), and a land card there still can enter, by being played or put onto the battlefield.
fn no_nonland_permanent_cards_from_graveyards_or_libraries(q: &dyn Query, _: &Asker, card: ObjectId) -> bool {
    !(matches!(q.obj(card).zone.kind(), ZoneKind::Graveyard | ZoneKind::Library) && is_nonland_permanent_card(&q.chars(card)))
}

pub static CARD: CardDef = CardDef {
    enter_rules: &[no_nonland_permanent_cards_from_graveyards_or_libraries],
    cast_rules: &[players_cant_cast_from_graveyards_or_libraries],
    ..oracle::WEATHERED_RUNESTONE
};
