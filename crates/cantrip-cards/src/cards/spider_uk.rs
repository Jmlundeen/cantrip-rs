use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::{effects, history, web_slinging};
use cantrip_oracle as oracle;

/// "If two or more creatures entered the battlefield under your control this turn" (CR 603.4, 603.6a): asked of the event history,
/// each creature as it entered under the controller it entered under, tokens included, whatever has happened to it since. An
/// intervening "if": checked as the end step begins and again as the ability resolves.
fn two_or_more_creatures_entered_under_your_control_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    history::creatures_entered_under_control_this_turn(q.history(), asker.controller) >= 2
}

/// "Draw a card and gain 2 life" (CR 121.1, 119.1): one sentence, so the draw and the life gain are done in turn.
fn draw_and_gain_2(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)?;
    g.end_sentence();
    effects::gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{2}{W}"), "Web-slinging {2}{W}")],
    abilities: &[triggered(
        YOUR_END_STEP,
        draw_and_gain_2,
        "At the beginning of your end step, if two or more creatures entered the battlefield under your control this turn, you draw a card and gain 2 life.",
    )
    .condition(two_or_more_creatures_entered_under_your_control_this_turn)],
    ..oracle::SPIDER_UK
};
