use cantrip_api::*;
use cantrip_mechanics::triggers::CAST_THIS_SPELL;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "When you cast this spell, copy it X times. You may choose new targets for the copies." (CR 707.10, 707.10c, 603.2): X is the
/// value announced for the spell, one copy at a time, the controller choosing new targets for each in turn. The copies are not cast,
/// so this trigger does not trigger for them (CR 707.10), and each has the same X (CR 107.3, 707.2).
fn copy_x_times(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g.obj(item.object).x;
    for _ in 0..x {
        g.copy_spell(item.object, item.controller, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::damage_first_target::<1>),
    abilities: &[triggered(CAST_THIS_SPELL, copy_x_times, "When you cast this spell, copy it X times. You may choose new targets for the copies.")],
    ..oracle::PHOTON_BLAST_BARRAGE
};
