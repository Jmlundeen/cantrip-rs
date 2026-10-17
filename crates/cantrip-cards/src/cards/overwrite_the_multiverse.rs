use cantrip_api::*;
use cantrip_mechanics::{effects, empower};
use cantrip_oracle as oracle;

/// "Exile all creatures. Empower Jace X, where X is the number of creatures exiled this way." The creatures are every creature on the
/// battlefield, any controller's, a token too, exiled together (CR 406.2, 101.4); X counts the ones that were exiled, so a creature
/// an effect kept from being exiled does not count. With X of 0 the empower still makes the Jace token if there is none, and it has no
/// loyalty and goes to the graveyard (CR 704.5i, 701.71a), so nothing is left.
fn overwrite(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled = effects::exile_all_creatures(g);
    empower::jace(g, item.controller, item.object, exiled.min(i16::MAX as usize) as i16)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], overwrite),
    ..oracle::OVERWRITE_THE_MULTIVERSE
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
