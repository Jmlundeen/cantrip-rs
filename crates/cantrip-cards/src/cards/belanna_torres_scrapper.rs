use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

const MAY_REVEAL: Question = Question::new(
    "Reveal",
    "reveal the top card of your library?",
    "The card is revealed, and drawn if it is an artifact card",
    "Reveal it",
    "Don't reveal it",
);

/// "{2}, {T}: Surveil 2, then you may reveal the top card of your library. Draw a card if it's an artifact card. If you
/// don't draw a card this way, put two +1/+1 counters on B'Elanna Torres." (CR 701.25a, 701.20a, 121.1, 608.2c): the
/// surveil is resolved first, so the card revealed is the one it left on top. The revealing is the controller's choice
/// (CR 608.2d), and every way of not drawing - declining, a nonartifact card, an empty library - puts the counters on
/// (CR 608.2c). A replacement effect that made the draw draw nothing counts as not drawing this way (CR 614.6).
fn surveil_then_reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 2)?;
    if !effects::reveal_top_and_draw_if_artifact(g, item, Some(&MAY_REVEAL))? {
        g.add_counters(item.object, CounterKind::PlusOne, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}, {T}"),
        surveil_then_reveal,
        "{2}, {T}: Surveil 2, then you may reveal the top card of your library. Draw a card if it's an artifact card. If you don't draw a card this way, put two +1/+1 counters on B'Elanna Torres.",
    )],
    ..oracle::BELANNA_TORRES_SCRAPPER
};
