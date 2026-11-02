use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::PUT_INTO_YOUR_GRAVEYARD_FROM_YOUR_LIBRARY;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("PutNarcomoebaOntoBattlefield", "put Narcomoeba onto the battlefield?", "It was put into your graveyard from your library", "Put it onto the battlefield", "Leave it");

/// "When this card is put into your graveyard from your library, you may put it onto the battlefield." (CR 608.2, 400.7):
/// nothing is asked once the card is no longer the object that reached the graveyard.
fn put_onto_battlefield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::ask_may_this_card(g, item, Purpose::Ask(&ASK))? {
        g.put_onto_battlefield(item.object, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        PUT_INTO_YOUR_GRAVEYARD_FROM_YOUR_LIBRARY,
        put_onto_battlefield,
        "When this card is put into your graveyard from your library, you may put it onto the battlefield.",
    )],
    ..oracle::NARCOMOEBA
};
