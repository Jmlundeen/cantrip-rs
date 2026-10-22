use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::{choose, haste_until_end_of_turn, library, top_of_library};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("PutOntoBattlefield", "put the revealed creature card onto the battlefield?", "It gains haste until end of turn. If you don't, it goes into your hand", "Put it onto the battlefield", "Put it into your hand");

/// "Look at the top six cards of your library. You may reveal a creature card from among them. If that card has mana value
/// 2 or less, you may put it onto the battlefield and it gains haste until end of turn. If you didn't put the revealed card
/// onto the battlefield this way, put it into your hand. Put the rest on the bottom of your library in a random order."
/// A smaller library shows what it has (CR 609.3). The revealed card is judged by its mana value in the library (CR 202.3),
/// and is put into the hand unless it really arrived on the battlefield. Every other card, and nothing else, goes to the
/// bottom in a random order (CR 401.4).
fn break_out(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let looked = top_of_library(g, you, 6);
    g.look_at(you, &looked);
    let creatures: Vec<ObjectId> = looked.iter().copied().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::Reveal, "creature", looked.clone(), creatures, 1, true)?;
    if let Some(&card) = chosen.first() {
        g.reveal(you, &[card]);
        let mut on_battlefield = false;
        if g.mana_value(card) <= 2 && ask_may(g, you, item.object, Purpose::Ask(&ASK))? {
            let entered = g.put_onto_battlefield(card, you, false)?;
            // The effect gives haste only to a permanent that is on the battlefield; a card that a replacement effect
            // sent elsewhere (CR 614) was not put there, so it goes into the hand below.
            haste_until_end_of_turn(g, item.object, entered);
            on_battlefield = g.is_on_battlefield(entered);
        }
        if !on_battlefield {
            g.return_to_hand_revealed(&[card]);
        }
    }
    library::rest_to_bottom_in_random_order(g, you, &looked);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], break_out),
    ..oracle::BREAK_OUT
};
