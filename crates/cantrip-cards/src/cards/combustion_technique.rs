use cantrip_api::*;
use cantrip_mechanics::replacements::{EXILE_INSTEAD_OF_DYING, exile_if_it_would_die};
use cantrip_mechanics::{damage_target, learn, targets};
use cantrip_oracle as oracle;

/// "Combustion Technique deals damage equal to 2 plus the number of Lesson cards in your graveyard to target creature. If that
/// creature would die this turn, exile it instead." The Lessons are counted as the spell resolves (CR 608.2c), in its
/// controller's graveyard; the spell is on the stack, not among them. The replacement is the spell's own effect on that creature
/// for the turn (CR 611.2c, 614.1a), for any way it would die, made whether or not the damage is dealt (the ruling), as Suplex's is.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let lessons = learn::lesson_cards_in_graveyard(g, item.controller);
    exile_if_it_would_die(g, item.object, 0, creature);
    damage_target(g, item, 0, 2 + lessons as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    granted_replacements: &[EXILE_INSTEAD_OF_DYING],
    ..oracle::COMBUSTION_TECHNIQUE
};
