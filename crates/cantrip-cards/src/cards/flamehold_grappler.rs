use cantrip_api::*;
use cantrip_mechanics::{copy, delayed, triggers::{self, THIS_ENTERS}};
use cantrip_oracle as oracle;

/// "When this creature enters, copy the next spell you cast this turn when you cast it" (CR 603.7b, 707.10): sets up a delayed
/// triggered ability, number 0 of the card's `delayed` list, that triggers once on the next spell its controller casts this turn
/// and ends with the turn if none is cast.
fn watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::next_this_turn(g, item, 0, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        watch,
        "When this creature enters, copy the next spell you cast this turn when you cast it. You may choose new targets for the copy. (A copy of a permanent spell becomes a token.)",
    )],
    delayed: &[triggered(
        when_asked(EventMask::SPELL_CAST, triggers::you_cast_a_spell_about_it),
        copy::copy_the_cast_spell,
        "Copy the next spell you cast this turn when you cast it. You may choose new targets for the copy.",
    )],
    ..oracle::FLAMEHOLD_GRAPPLER
};
