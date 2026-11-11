use cantrip_api::*;
use cantrip_mechanics::history::entered_this_turn;
use cantrip_oracle as oracle;

/// "{2}, {T}: Draw a card for each nontoken permanent you control that entered this turn." The permanents are counted as the
/// ability resolves: the event history says which of the controller's permanents entered this turn (CR 603.6a, 400.7: a
/// permanent that left and came back is a new object that entered again), a token of their own is not counted, and neither
/// is anything an opponent controls. That many cards are drawn, one at a time (CR 121.2).
fn draw_for_new_permanents(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let newcomers = g
        .permanents_of(you)
        .into_iter()
        .filter(|&id| !g.state().obj(id).token && entered_this_turn(g.history(), id))
        .count() as u32;
    g.draw(you, newcomers);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}").and(TAP),
        draw_for_new_permanents,
        "{2}, {T}: Draw a card for each nontoken permanent you control that entered this turn.",
    )],
    ..oracle::SPOCK_VOICE_OF_REASON
};
