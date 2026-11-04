use cantrip_api::*;
use cantrip_mechanics::{effects, exchange, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

const INVERT: Question = Question::new(
    "Darkforce Inversion",
    "exchange life totals with the opponent?",
    "If you lost life this way, draw that many cards",
    "Exchange",
    "Don't exchange",
);

/// "You may exchange life totals with target opponent. If you lost life this way, draw that many cards" (CR 701.12c, 121.1):
/// nothing is exchanged if the target is no longer legal (CR 608.2b), the controller declines, or the exchange can't be
/// completed (CR 701.12a); the cards drawn are the life the controller actually lost, which a replacement effect may have
/// changed.
fn darkforce_inversion(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    if !effects::ask_may(g, item.controller, item.object, Purpose::Ask(&INVERT))? {
        return Ok(());
    }
    let lost = exchange::life_totals(g, item.controller, opponent);
    g.draw(item.controller, lost);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        darkforce_inversion,
        "Darkforce Inversion — When Mister Negative enters, you may exchange life totals with target opponent. If you lost life this way, draw that many cards.",
    )
    .targets(&[targets::OPPONENT])],
    ..oracle::MISTER_NEGATIVE
};
