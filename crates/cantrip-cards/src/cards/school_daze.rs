use cantrip_api::*;
use cantrip_mechanics::{counter_target, effects, targets};
use cantrip_oracle as oracle;

/// Mode 2: "Counter target spell (CR 701.6). Draw a card." The spell is countered and its controller draws, one instruction
/// after the other (CR 608.2c). If the target is no longer legal the spell does not resolve at all (CR 608.2b): no draw.
fn counter_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    effects::draw(g, item, 1)
}

/// "Choose one" (CR 700.2).
static MODAL: ModalDef = modes(&[
    mode(|g, item| effects::draw(g, item, 3), "Do Homework — Draw three cards."),
    mode(counter_and_draw, "Fight Crime — Counter target spell. Draw a card.").targets(&[targets::SPELL]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SCHOOL_DAZE };
