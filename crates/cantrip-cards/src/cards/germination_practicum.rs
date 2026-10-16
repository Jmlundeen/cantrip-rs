use cantrip_api::*;
use cantrip_mechanics::paradigm;
use cantrip_oracle as oracle;

/// Put two +1/+1 counters on each creature you control.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    paradigm: Some(paradigm::SPELL_ABILITY),
    delayed: &[paradigm::TRIGGER],
    ..oracle::GERMINATION_PRACTICUM
};
