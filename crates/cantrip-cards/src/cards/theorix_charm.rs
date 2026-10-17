use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, pump, targets};
use cantrip_oracle as oracle;

/// "Counter target noncreature spell unless its controller pays {2}." (CR 118.12)
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{2}"))?;
    Ok(())
}

/// "Target creature gets -2/-2 until end of turn." Nothing if the target is no longer legal (CR 608.2b).
fn shrink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, -2, -2);
    }
    Ok(())
}

/// "Mill three cards, then draw a card." (CR 701.17a, 121.1): the draw happens whatever the mill found, an empty
/// library included.
fn mill_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 3);
    g.draw(item.controller, 1);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(counter, "Counter target noncreature spell unless its controller pays {2}.").targets(&[targets::NONCREATURE_SPELL]),
    mode(shrink, "Target creature gets -2/-2 until end of turn.").targets(&[targets::CREATURE]),
    mode(mill_and_draw, "Mill three cards, then draw a card."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::THEORIX_CHARM };
