use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Put target creature into its owner's library second from the top. Its controller gains 3 life: the controller it had
/// when the spell resolved (CR 608.2h), asked before the creature leaves.
fn oust(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(creature);
    g.put_into_library(&[creature], LibraryPlace::FromTop(1));
    g.gain_life(controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], oust),
    ..oracle::OUST
};
