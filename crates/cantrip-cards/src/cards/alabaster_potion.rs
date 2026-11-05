use cantrip_api::*;
use cantrip_mechanics::{effects, prevention, targets};
use cantrip_oracle as oracle;

/// Prevent the next X damage to the chosen target this turn (CR 615.7).
fn shield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::next_damage_to_target(g, item, 0, 0, u32::from(item.x));
    Ok(())
}

static MODES: ModalDef = modes(&[
    mode(effects::target_player_gains_x_life, "Target player gains X life.").targets(&[targets::PLAYER]),
    mode(shield, "Prevent the next X damage that would be dealt to any target this turn.").targets(&[targets::ANY]),
]).choose(1, 1);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODES),
    granted_preventions: &[prevention::ANY_DAMAGE.text("Prevent the next X damage that would be dealt to any target this turn.")],
    ..oracle::ALABASTER_POTION
};
