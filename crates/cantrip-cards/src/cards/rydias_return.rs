use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Return up to two target permanent cards from your graveyard to your hand": the ones still legal, together (CR 608.2b).
fn return_permanent_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = targets::legal_objects(g, item);
    g.return_to_hand(&cards);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::creatures_you_control_get::<3, 3>, "Creatures you control get +3/+3 until end of turn."),
    mode(return_permanent_cards, "Return up to two target permanent cards from your graveyard to your hand.")
        .targets(&[up_to(2, targets::PERMANENT_CARD_IN_YOUR_GRAVEYARD.filter)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::RYDIAS_RETURN };
