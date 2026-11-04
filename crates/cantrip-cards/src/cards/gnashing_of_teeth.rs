use cantrip_api::*;
use cantrip_mechanics::replacements::{EXILE_INSTEAD_OF_DYING, exile_if_it_would_die};
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// Mode 1: "Target creature gets -5/-5 until end of turn. If that creature would die this turn, exile it instead." The
/// replacement is the spell's own effect on that creature (CR 611.2c, 614.1a), made after the -5/-5 and before any
/// state-based action could put it into a graveyard, so it covers the death that -5/-5 causes too (CR 704.3).
fn shrink_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::target_creature_gets::<-5, -5>(g, item)?;
    g.end_sentence();
    exile_if_it_would_die(g, item, 0, creature);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(shrink_and_exile, "Target creature gets -5/-5 until end of turn. If that creature would die this turn, exile it instead.")
        .targets(&[targets::CREATURE]),
    mode(effects::creatures_target_player_controls_get::<0, -1, -1>, "Creatures target player controls get -1/-1 until end of turn.").targets(&[targets::PLAYER]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    granted_replacements: &[EXILE_INSTEAD_OF_DYING.text("If that creature would die this turn, exile it instead.")],
    ..oracle::GNASHING_OF_TEETH
};
