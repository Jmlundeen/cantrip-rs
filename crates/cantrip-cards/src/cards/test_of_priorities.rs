use cantrip_api::*;
use cantrip_mechanics::{delayed, effects, targets};
use cantrip_oracle as oracle;

/// "Surveil 2, then draw a card" (CR 701.25, 608.2c): the surveil is complete, its own triggers waiting, before the draw.
fn surveil_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 2)?;
    g.draw(item.controller, 1);
    Ok(())
}

/// "Choose one —" (CR 700.2). The second mode's "Exile target creature you control. Return that card to the battlefield
/// under its owner's control at the beginning of the next end step" (CR 701.13a, 603.7c, 400.7) is the shared slot-0 body:
/// the delayed ability is about the card it became in exile.
static MODAL: ModalDef = modes(&[
    mode(surveil_then_draw, "Power to Engines \u{2014} Surveil 2, then draw a card."),
    mode(delayed::exile_target_then_return_at_next_end_step_0, "Power to Life Support \u{2014} Exile target creature you control. Return that card to the battlefield under its owner's control at the beginning of the next end step.")
        .targets(&[targets::CREATURE_YOU_CONTROL]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    delayed: &[delayed_trigger(
        delayed::return_subject_to_battlefield,
        "Return that card to the battlefield under its owner's control at the beginning of the next end step.",
    )],
    ..oracle::TEST_OF_PRIORITIES
};
