use cantrip_api::*;
use cantrip_mechanics::replacements::{EXILE_INSTEAD_OF_DYING, exile_if_it_would_die};
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Essence Burn deals 5 damage to target black or green creature or planeswalker. If that permanent would die this turn,
/// exile it instead." The replacement is the spell's own effect on that permanent (CR 611.2c, 614.1a), made whether or not
/// the damage is dealt, and before it, so it covers every way the permanent would die this turn (CR 700.4).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        exile_if_it_would_die(g, item.object, 0, permanent);
        damage_target(g, item, 0, 5);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::creature_or_planeswalker_of_either_color::<{ Color::Black as u8 }, { Color::Green as u8 }>()], effect),
    granted_replacements: &[EXILE_INSTEAD_OF_DYING],
    ..oracle::ESSENCE_BURN
};
