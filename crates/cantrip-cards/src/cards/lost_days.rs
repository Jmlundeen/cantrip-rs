use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "The owner of target creature or enchantment puts it into their library second from the top or on the bottom. You create a Clue
/// token." With its one target gone or illegal the spell does not resolve at all (CR 608.2b: the core removes it from the stack), so no
/// Clue is made. Creating a Clue is not investigating.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    effects::owner_puts_into_library_second_from_top_or_on_the_bottom(g, item, permanent)?;
    tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(targets::creature_or_enchantment)], effect),
    labels: &["Second from the top", "On the bottom"],
    ..oracle::LOST_DAYS
};

/// The Clue token it makes.
pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
