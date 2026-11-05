use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, effects, pump, targets};
use cantrip_oracle as oracle;

/// "Target Klingon": a permanent of the creature type, read through the words of this card's text (CR 115.1, 612.2).
fn target_klingon(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_of_any_subtype(q, asker, t, &[Subtype::Klingon])
}

/// "I Am Klingon — Target Klingon gets +1/+1 and gains double strike until end of turn." (CR 611.2, 702.4): one
/// instruction, so a single effect grants both.
fn klingon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 1, 1);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::DoubleStrike);
    });
    Ok(())
}

/// "I Am Starfleet — Discard a card. If you do, draw two cards." (CR 701.9): the discard is asked of any hand that is not
/// empty, and the draw follows only when a card was discarded.
fn starfleet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !discard_chosen(g, item.controller, item.object, 1)?.is_empty() {
        g.draw(item.controller, 2);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(klingon, "I Am Klingon \u{2014} Target Klingon gets +1/+1 and gains double strike until end of turn.")
        .targets(&[target(target_klingon)]),
    mode(starfleet, "I Am Starfleet \u{2014} Discard a card. If you do, draw two cards."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::TEST_OF_IDENTITY };
