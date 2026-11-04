use cantrip_api::*;
use cantrip_mechanics::{counter_target, effects, targets};
use cantrip_oracle as oracle;

/// Mode 1: "Counter target spell" (CR 701.6). Nothing happens if it is no longer a legal target (CR 608.2b).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

/// "Choose one or both" (CR 700.2): the modes are carried out in the order printed (CR 608.2c).
static MODAL: ModalDef = modes(&[
    mode(counter, "Counter target spell.").targets(&[targets::SPELL]),
    mode(effects::tap_targets, "Tap one or two target creatures.").targets(&[targets::CREATURE.between(1, 2)]),
])
.choose(1, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::AMAZING_ACROBATICS };
