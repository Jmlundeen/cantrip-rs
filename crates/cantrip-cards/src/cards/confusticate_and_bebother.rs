use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, effects, targets};
use cantrip_oracle as oracle;

/// "Counter target spell unless its controller pays {4}." (CR 118.12, 701.6)
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{4}"))?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(counter, "Counter target spell unless its controller pays {4}.").targets(&[targets::SPELL]),
    mode(effects::draw_two_then_discard, "Draw two cards, then discard a card."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::CONFUSTICATE_AND_BEBOTHER };
