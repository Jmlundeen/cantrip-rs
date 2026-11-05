use cantrip_api::*;
use cantrip_mechanics::{effects, library::exile_top_play_until_end_of_next_turn, targets};
use cantrip_oracle as oracle;

/// "Exile the top two cards of your library. Until the end of your next turn, you may play those cards." (Wrenn's Resolve and
/// Encounter the Unknown's first mode say it with this same body.) A land among them is the turn's land play (CR 305.1).
fn exile_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 2);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(exile_and_play, "Exile the top two cards of your library. Until the end of your next turn, you may play those cards."),
    mode(effects::targets_get::<2, 1>, "Up to two target creatures each get +2/+1 until end of turn.").targets(&[targets::CREATURE.between(0, 2)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SEIZE_OPPORTUNITY };
