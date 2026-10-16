use cantrip_api::*;
use cantrip_mechanics::replacements::{EXILE_INSTEAD_OF_DYING, exile_if_it_would_die};
use cantrip_mechanics::{bargain, scry, targets};
use cantrip_oracle as oracle;

/// "Torch the Tower deals 2 damage to target creature or planeswalker. If this spell was bargained, instead it deals 3
/// damage to that permanent and you scry 1. If a permanent dealt damage by Torch the Tower would die this turn, exile it
/// instead." The replacement is the spell's own effect (CR 611.2c, 614.1a) on a permanent it dealt damage to: none when
/// the damage was prevented, and for any way the permanent would die, not only lethal damage (CR 700.4). With its one
/// target illegal the spell does not resolve (CR 608.2b), so there is no scry either.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let bargained = bargain::bargained(item);
    if g.damage(item.object, Target::Object(permanent), if bargained { 3 } else { 2 }) > 0 {
        exile_if_it_would_die(g, item.object, 0, permanent);
    }
    if bargained {
        scry(g, item.controller, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    optional_cost: Some(bargain::OPTIONAL_COST),
    labels: bargain::LABELS,
    granted_replacements: &[EXILE_INSTEAD_OF_DYING],
    ..oracle::TORCH_THE_TOWER
};
