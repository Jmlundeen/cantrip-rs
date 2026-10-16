use cantrip_api::*;
use cantrip_mechanics::{choose, goyf, triggers, effects};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Mill", "mill that many cards?", "If you do, you may put a creature card from among them into your hand", "Mill", "Don't mill");

/// You may mill that many cards; if you do, you may put a creature card from among them into your hand (CR 701.17c).
fn mill_and_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    let player = item.controller;
    let among = g.mill(player, item.data[0]);
    let eligible: Vec<ObjectId> = among.iter().copied().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    let chosen = choose::cards_from(g, player, item.object, Purpose::PutIntoHand, among, eligible, 1, true)?;
    g.return_to_hand(&chosen);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT, mill_and_take, "Whenever this creature deals combat damage to a player, you may mill that many cards. If you do, you may put a creature card from among them into your hand.")],
    statics: &[goyf::SIZE],
    ..oracle::BARROWGOYF
};
