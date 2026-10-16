use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::{counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {3} (CR 118.12).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{3}"))?;
    Ok(())
}

/// Choose three; you may choose the same mode more than once (CR 700.2d).
static MODAL: ModalDef = modes(&[
    mode(counter, "Counter target spell unless its controller pays {3}.")
        .targets(&[targets::SPELL]),
    mode(effects::return_first_target_to_hand, "Return target creature to its owner's hand.")
        .targets(&[targets::CREATURE]),
    mode(|g, item| effects::draw(g, item, 1), "Draw a card."),
])
    .choose(3, 3)
    .repeat();

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::MYSTIC_CONFLUENCE };
