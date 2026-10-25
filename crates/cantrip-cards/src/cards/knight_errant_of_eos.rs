use cantrip_api::*;
use cantrip_mechanics::select::{creature_cards, mana_value_at_most, up_to};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{choose, top_of_library};
use cantrip_oracle as oracle;

/// "When this creature enters, look at the top six cards of your library. You may reveal up to two creature cards with mana
/// value X or less from among them, where X is the number of creatures that convoked this creature. Put the revealed cards
/// into your hand, then shuffle." X counts the creatures that convoked the spell this permanent was (CR 702.51c, 608.3),
/// whether or not they are still on the battlefield (the 2023-04-14 ruling); a Knight that was not cast, or cast without
/// convoke, has X of 0. Each card's mana value is read in the library (CR 202.3). A smaller library shows what it has (CR
/// 609.3), and the library is shuffled whatever was taken (CR 701.24).
fn look_at_six(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let x = count_u32(g.history().convoked(item.object).count());
    let looked = top_of_library(g, you, 6);
    g.look_at(you, &looked);
    let revealed = choose::select(g, you, item.object, Purpose::Reveal, looked, creature_cards(up_to(2)).with(mana_value_at_most(x)))?;
    if !revealed.is_empty() {
        g.reveal(you, &revealed);
        g.return_to_hand_revealed(&revealed);
    }
    g.shuffle(you);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        look_at_six,
        "When this creature enters, look at the top six cards of your library. You may reveal up to two creature cards with mana value X or less from among them, where X is the number of creatures that convoked this creature. Put the revealed cards into your hand, then shuffle.",
    )],
    ..oracle::KNIGHT_ERRANT_OF_EOS
};
