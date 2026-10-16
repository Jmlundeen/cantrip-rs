use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Target permanent card in your graveyard that was put there from anywhere this turn" (CR 115.1, 608.2b): a card
/// arrives in a graveyard as a new object, made by a move the event history records.
fn permanent_card_put_there_this_turn(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_card_in_your_graveyard(q, asker, t) && matches!(t, Target::Object(id) if cantrip_mechanics::history::arrived_this_turn(q, id))
}

fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, return_it, "When Night Nurse enters, choose target permanent card in your graveyard that was put there from anywhere this turn. Return it to your hand.")
        .targets(&[target(permanent_card_put_there_this_turn)])],
    ..oracle::NIGHT_NURSE_HEALER_OF_HEROES
};
