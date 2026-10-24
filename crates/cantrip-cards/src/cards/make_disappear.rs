use cantrip_api::*;
use cantrip_mechanics::{casualty, counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// "Counter target spell unless its controller pays {2}" (CR 118.12, 701.6): the spell's controller is asked, and may make
/// mana for it while asked. Nothing if the target spell is gone (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{2}"))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    optional_cost: Some(casualty::optional::<1>()),
    labels: &["Cast without casualty", "Casualty 1: sacrifice a creature with power 1 or greater"],
    abilities: &[casualty::trigger("Casualty 1 (As you cast this spell, you may sacrifice a creature with power 1 or greater. When you do, copy this spell and you may choose a new target for the copy.)")],
    ..oracle::MAKE_DISAPPEAR
};
