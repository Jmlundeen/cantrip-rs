use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

/// "Target player gains 3 life" (CR 119.3), if the player is still a legal target (CR 608.2b).
fn gains_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        g.gain_life(player, 3);
    }
    Ok(())
}

/// "Prevent the next 3 damage that would be dealt to any target this turn" (CR 615.7): a shield of 3 on the creature,
/// player, planeswalker or battle, used up 1 for each 1 damage it prevents, from any source, until end of turn.
fn shields_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::next_damage_to_target(g, item, 0, 0, 3);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(gains_three, "Target player gains 3 life.").targets(&[targets::PLAYER]),
    mode(shields_three, "Prevent the next 3 damage that would be dealt to any target this turn.").targets(&[targets::ANY]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    granted_preventions: &[prevention::ANY_DAMAGE],
    ..oracle::HEALING_SALVE
};
