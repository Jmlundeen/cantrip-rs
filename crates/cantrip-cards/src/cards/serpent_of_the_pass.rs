use cantrip_api::*;
use cantrip_mechanics::learn;
use cantrip_oracle as oracle;

/// "If there are three or more Lesson cards in your graveyard, you may cast this spell as though it had flash" (CR 702.8, 307.5):
/// asked of the player who would cast it as the casting begins, counting only their own graveyard (CR 404.1) and not again
/// afterwards.
fn flash_with_three_lessons(q: &dyn Query, asker: &Asker) -> bool {
    learn::lesson_cards_in_graveyard(q, asker.controller) >= 3
}

/// "This spell costs {1} less to cast for each noncreature, nonland card in your graveyard" (CR 601.2f, 118.7a): counted as
/// the cost is determined, the cards as they are in the caster's graveyard; a Kindred noncreature card counts. The reduction
/// never takes off the {U}{U} (CR 118.7a, the rulings), which the core leaves in place. Its mana value stays 7 (CR 118.7).
fn less_per_noncreature_nonland_card_in_your_graveyard(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    let cards = q.state().zone(Zone::graveyard(player)).iter().filter(|&&id| !q.chars(id).is(CardType::Creature) && !q.chars(id).is(CardType::Land)).count();
    CostDelta::less(cards as u32)
}

pub static CARD: CardDef = CardDef {
    flash_if: Some(flash_with_three_lessons),
    own_cost_changes: &[less_per_noncreature_nonland_card_in_your_graveyard],
    ..oracle::SERPENT_OF_THE_PASS
};
