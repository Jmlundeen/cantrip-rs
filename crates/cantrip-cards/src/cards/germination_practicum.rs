use cantrip_api::*;
use cantrip_mechanics::paradigm;
use cantrip_oracle as oracle;

/// Put two +1/+1 counters on each creature you control.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for id in g.permanents_of(item.controller) {
        if g.chars(id).is(CardType::Creature) {
            g.add_counters(id, CounterKind::PlusOne, 2);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    paradigm: true,
    abilities: &[paradigm::TRIGGER],
    ..oracle::GERMINATION_PRACTICUM
};
