use cantrip_api::*;
use cantrip_mechanics::{effects, library::exile_top_play_until_end_of_next_turn};
use cantrip_oracle as oracle;

/// "Exile the top two cards of your library. Until the end of your next turn, you may play those cards."
/// The permission lasts through the controller's next turn, including lands under normal land-play timing (CR 305.1).
fn come_in_peace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 2);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(come_in_peace, "Come in Peace — Exile the top two cards of your library. Until the end of your next turn, you may play those cards."),
    mode(effects::damage_each_creature_and_player::<2>, "Shoot to Kill — Encounter the Unknown deals 2 damage to each creature and each player."),
])
.choose(1, 1);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::ENCOUNTER_THE_UNKNOWN
};
