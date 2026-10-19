use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::NONCREATURE_SPELL_YOU_CAST;
use cantrip_oracle as oracle;

/// "Whenever you cast a noncreature spell, put a +1/+1 counter on this creature."
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_this(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        NONCREATURE_SPELL_YOU_CAST,
        counter,
        "Whenever you cast a noncreature spell, put a +1/+1 counter on this creature.",
    )],
    ..oracle::CHANDRAS_EMBERLING
};
