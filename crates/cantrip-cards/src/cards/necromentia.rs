use cantrip_api::*;
use cantrip_mechanics::{library, targets, tokens};
use cantrip_oracle as oracle;

pub static EXTRA: &[&CardDef] = &[&tokens::ZOMBIE];

/// "Choose a card name other than a basic land card name. Search target opponent's graveyard, hand, and library for any
/// number of cards with that name and exile them. That player shuffles, then creates a 2/2 black Zombie creature token for
/// each card exiled from their hand this way."
///
/// The name is chosen as the spell resolves (CR 608.2c), and the core refuses a basic land's (CR 201.4a, 205.4c). You
/// search the three zones in the order named, any number in each (CR 701.23b); the cards found are exiled together (CR
/// 406.2) and the opponent shuffles (CR 701.24). Then that player - not you - creates the tokens, one for each card that
/// reached exile from their hand, all at once (CR 701.7a). With the opponent no longer a legal target the spell doesn't
/// resolve (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let ask = RequestKind::ChooseName { source: item.object, looked_at: Vec::new(), suggestion: None, kind: NameKind::OTHER_THAN_BASIC_LAND };
    // A card has "that name" when it is the card name chosen; with no name chosen nothing has it.
    let names: Vec<&'static str> = match g.ask(you, ask)? { Answer::Name(Some(wanted)) => vec![g.card(wanted).name], _ => Vec::new() };
    let from_hand = library::exile_any_number_named_from_graveyard_hand_and_library(g, you, opponent, item.object, &names)?;
    tokens::create(g, opponent, &tokens::ZOMBIE, from_hand)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::NECROMENTIA
};
