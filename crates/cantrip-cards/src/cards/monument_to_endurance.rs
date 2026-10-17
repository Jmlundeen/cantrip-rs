use cantrip_api::*;
use cantrip_mechanics::{effects, tokens, triggers};
use cantrip_oracle as oracle;

/// "Choose one that hasn't been chosen this turn" (CR 700.2): whether mode `mode` has not been chosen this turn by this
/// permanent, as its ability went on the stack. A mode chosen for an ability that was then countered stays chosen, and one chosen
/// by another Monument to Endurance is chosen by that one: each tracks its own, as an object that left and came back is a new
/// one (CR 400.7). With all three chosen the ability still triggers and is removed as it would be put on the stack (CR 603.3c).
fn mode_not_chosen_this_turn(q: &dyn Query, asker: &Asker, mode: u8) -> bool {
    !q.history().this_turn().iter().any(|r| {
        matches!(r.event, Event::ModesChosen { source, modes, .. } if source == asker.source && modes & (1 << mode) != 0)
    })
}

/// "Each opponent loses 3 life."
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 3);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(|g, item| effects::draw(g, item, 1), "Draw a card."),
    mode(tokens::create_a_treasure, "Create a Treasure token."),
    mode(drain, "Each opponent loses 3 life."),
])
.allowed(mode_not_chosen_this_turn);

/// The Treasure token.
pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        triggers::YOU_DISCARD,
        &MODAL,
        "Whenever you discard a card, choose one that hasn't been chosen this turn \u{2014} draw a card; or create a Treasure token; or each opponent loses 3 life.",
    )],
    ..oracle::MONUMENT_TO_ENDURANCE
};
