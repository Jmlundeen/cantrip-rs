use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast for each instant and sorcery card in your graveyard" (CR 601.2f, 118.7a): counted as
/// the cost is determined, the cards as they are in the graveyard.
fn less_per_instant_and_sorcery_in_your_graveyard(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    // Only the cost of casting it: it has no abilities to activate, and "this spell" is a spell.
    let spell = matches!(costed, Costed::Spell(_));
    let cards = q
        .state()
        .zone(Zone::graveyard(player))
        .iter()
        .filter(|&&id| spell && (q.chars(id).is(CardType::Instant) || q.chars(id).is(CardType::Sorcery)))
        .count();
    CostDelta::less(cards.min(255) as u8)
}

/// "Tap up to two target creatures": each target still legal is tapped (CR 608.2b).
fn tap_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for creature in targets::legal_objects(g, item) {
        g.tap(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[less_per_instant_and_sorcery_in_your_graveyard],
    enters: Some(cantrip_mechanics::mana::enters_tapped_unless_your_turn),
    abilities: &[triggered(THIS_ENTERS, tap_them, "When this creature enters, tap up to two target creatures.")
        .targets(&[targets::CREATURE.between(0, 2)])],
    ..oracle::EDDYMURK_CRAB
};
