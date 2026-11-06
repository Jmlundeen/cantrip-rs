use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, learn};
use cantrip_oracle as oracle;

/// "If there are three or more Lesson cards in your graveyard, you may cast this spell as though it had flash" (CR 702.8, 307.5):
/// asked of the player who would cast it as the casting begins, counting only their own graveyard (CR 404.1) and not again
/// afterwards.
fn flash_with_three_lessons(q: &dyn Query, asker: &Asker) -> bool {
    learn::lesson_cards_in_graveyard(q, asker.controller) >= 3
}

pub static CARD: CardDef = CardDef {
    flash_if: Some(flash_with_three_lessons),
    own_cost_changes: &[cost_reduction::for_each_noncreature_nonland_card_in_your_graveyard],
    ..oracle::SERPENT_OF_THE_PASS
};
