use cantrip_api::*;
use cantrip_mechanics::{copy, graveyards};
use cantrip_oracle as oracle;

const ASK: Question = Question::new(
    "CopyAsEnters",
    "enter as a copy of a creature card in a graveyard?",
    "ChoH'a': he is a Klingon Spy in addition to his other types and has menace. If you don't, he enters as himself",
    "Choose a card",
    "Enter as himself",
);

/// "ChoH'a' — You may have Voq enter as a copy of a creature card in a graveyard, except he's a Klingon Spy in addition to his
/// other types and has menace." A copy as he enters (CR 614.1c, 707.1), chosen rather than targeted (CR 608.2d) - an opponent's
/// graveyard as available as your own, and his own card not a candidate (CR 707.2) - with the exceptions applied to the copy.
fn choa(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    let entering = entry.object;
    let cards: Vec<ObjectId> = graveyards::cards_in_graveyards(g.state())
        .filter(|&id| id != entering && g.chars(id).is(CardType::Creature))
        .collect();
    copy::enter_as_copy_of_one_of(g, entry, &cards, |_, copy| copy, &ASK)?;
    let words = g.words(entering);
    entry.copy = entry.copy.map(|copy| {
        copy.with_subtype(words.subtype(Subtype::Klingon)).with_subtype(words.subtype(Subtype::Spy)).with_keyword(Keyword::Menace)
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(choa), ..oracle::VOQ_TORCHBEARER };
