use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Create a token that's a copy of target creature, except it's not legendary and it's a `size`/`size` `kind` in addition to
/// its other types" (CR 707.2, 707.9b, 111.1): the exceptions are part of the copy's copiable values, so a copy of the token is
/// a legendary-less `size`/`size` `kind` too. Nothing if the target is no longer legal (CR 608.2b).
fn copy_as(g: &mut dyn Game, item: &StackItem, kind: Subtype, size: i16) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(creature).nonlegendary().with_subtype(g.words(item.object).subtype(kind)).sized(size, size);
    g.create_token_copies_of(values, item.controller, 1)?;
    Ok(())
}

/// "Create a token that's a copy of target creature you control, except it's not legendary and it's a 4/4 Hero in addition to its other types."
fn hero(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    copy_as(g, item, Subtype::Hero, 4)
}

/// "Create a token that's a copy of target creature an opponent controls, except it's not legendary and it's a 2/2 Coward in addition to its other types."
fn coward(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    copy_as(g, item, Subtype::Coward, 2)
}

/// "Choose one" (CR 700.2): each mode has its own target (CR 700.2c).
static MODAL: ModalDef = modes(&[
    mode(hero, "Create a token that's a copy of target creature you control, except it's not legendary and it's a 4/4 Hero in addition to its other types.")
        .targets(&[targets::CREATURE_YOU_CONTROL]),
    mode(coward, "Create a token that's a copy of target creature an opponent controls, except it's not legendary and it's a 2/2 Coward in addition to its other types.")
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::EMBER_ISLAND_PRODUCTION };
