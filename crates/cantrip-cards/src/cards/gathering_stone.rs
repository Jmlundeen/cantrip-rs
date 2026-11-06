use cantrip_api::*;
use cantrip_mechanics::chosen_type::enters_choosing_a_creature_type;
use cantrip_mechanics::library;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_YOUR_UPKEEP;
use cantrip_oracle as oracle;

const REVEAL: Question = Question::new(
    "RevealChosenTypeToHand",
    "reveal it and put it into your hand?",
    "It is a card of the chosen type. If you don't, you may put it into your graveyard instead",
    "Reveal it",
    "Don't",
);

/// "Spells you cast of the chosen type cost {1} less to cast" (CR 601.2f, 205.3): only its controller's spells, judged on the
/// spell's own subtypes as it is cast; a generic reduction takes only the generic part (CR 118.7a, the 2025-11-17 ruling).
/// With no type chosen (a copy that never entered) nothing is of it.
fn chosen_type_spells_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    match q.state().chosen_subtype(q.text_linked(source)) {
        Some(chosen) if q.chars(spell).has_subtype(chosen) => CostDelta::less(1),
        _ => CostDelta::NONE,
    }
}

/// "Look at the top card of your library. If it's a card of the chosen type, you may reveal it and put it into your hand. If
/// you don't put the card into your hand, you may put it into your graveyard." (CR 701.20a.) It may be left unrevealed even if it
/// is of the chosen type (the 2025-11-17 ruling).
fn look_at_the_top_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let asker = g.item_asker(item, item.controller);
    library::look_at_top_may_take_else_bin(g, item, &REVEAL, |g, top| {
        g.state().chosen_subtype(g.linked_to(&asker)).is_some_and(|chosen| g.chars(top).has_subtype(chosen))
    })
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_choosing_a_creature_type),
    cost_changes: &[chosen_type_spells_cost_less],
    abilities: &[triggered(
        THIS_ENTERS_OR_YOUR_UPKEEP,
        look_at_the_top_card,
        "When this artifact enters and at the beginning of your upkeep, look at the top card of your library. If it's a card of the chosen type, you may reveal it and put it into your hand. If you don't put the card into your hand, you may put it into your graveyard.",
    )],
    ..oracle::GATHERING_STONE
};
