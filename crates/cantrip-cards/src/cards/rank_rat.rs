use cantrip_api::*;
use cantrip_mechanics::{each_discards_a_card, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, each opponent discards a card." Each opponent chooses in turn order and the cards are
/// discarded together (CR 101.4, 701.9); an opponent with no card in hand simply does not discard.
fn each_opponent_discards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let opponents: Vec<PlayerId> = g.state().apnap().filter(|&p| p != you).collect();
    each_discards_a_card(g, &opponents, item.object)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, each_opponent_discards, "When this creature enters, each opponent discards a card.")],
    ..oracle::RANK_RAT
};
