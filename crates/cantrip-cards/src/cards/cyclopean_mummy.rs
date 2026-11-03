use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_DIES};
use cantrip_oracle as oracle;

/// When this creature dies, exile each card it became in the graveyard (CR 400.7e, 730.3c).
fn exile_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::exile_what_it_became(g, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_DIES, exile_it, "When this creature dies, exile it.")],
    ..oracle::CYCLOPEAN_MUMMY
};
