use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Look at the top X cards of your library. Put one of them into your hand and the rest on the bottom of your library in a
/// random order." (CR 107.3, 401.4): X is the spell's as this ability resolves (the trigger's subject): its X on the stack, which an effect can
/// have changed since it was cast, or as it last was there once the spell has left it (CR 608.2h). With X of 0 nothing is
/// looked at; a library with fewer cards shows what it has (CR 609.3).
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    let state = g.state();
    let x = state.stack.iter().find(|i| i.kind == StackKind::Spell && i.object == spell).or_else(|| state.left_stack(spell)).map_or(0, |i| i.x);
    look_at_top(g, item.controller, item.object, x as usize, (1, 1), false, |_| true, Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_CAST_A_SPELL_WITH_X_IN_ITS_MANA_COST,
        look,
        "Whenever you cast a spell with {X} in its mana cost, look at the top X cards of your library. Put one of them into your hand and the rest on the bottom of your library in a random order.",
    )],
    ..oracle::GEOMETERS_ARTHROPOD
};
