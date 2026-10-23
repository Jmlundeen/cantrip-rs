use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::{empower, targets};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on up to one target creature. It gains vigilance until end of turn. Empower Jace 4."
fn counter_vigilance_and_empower(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Vigilance);
    }
    empower::jace(g, item.controller, item.object, 4)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE.optional()], counter_vigilance_and_empower),
    ..oracle::TAMS_RESISTANCE
};
