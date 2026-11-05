use cantrip_api::*;
use cantrip_mechanics::{copy, history};
use cantrip_oracle as oracle;

/// "You may have Ezri Dax enter as a copy of any creature card in your graveyard that was put there from the battlefield
/// this turn, except her name is Ezri Dax, Newly Joined and she's a legendary Trill in addition to her other types" (CR
/// 614.1c, 707.2, 707.9): a choice, not a target (CR 608.2d), with the exceptions becoming part of her copiable values (CR
/// 707.9b). Nothing to copy asks nothing and she enters as herself.
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    let you = entry.controller;
    // "A creature card in your graveyard that was put there from the battlefield this turn" (CR 400.7, 603.6c): a card milled,
    // discarded or countered was never on the battlefield, and one that left and came back is read by its latest move.
    let candidates: Vec<ObjectId> = history::put_into_graveyard_from_battlefield_this_turn(g, you).into_iter().filter(|&card| g.chars(card).is(CardType::Creature)).collect();
    copy::enter_as_copy_of_one_of(
        g,
        entry,
        &candidates,
        |_, values| values.named("Ezri Dax, Newly Joined").legendary().with_subtype(Subtype::Trill),
        &copy::COPY_AS_ENTERS,
    )
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(enter_as_copy), ..oracle::EZRI_DAX_NEWLY_JOINED };
