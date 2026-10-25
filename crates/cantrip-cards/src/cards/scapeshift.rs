use cantrip_api::*;
use cantrip_mechanics::{choose, library, select};
use cantrip_oracle as oracle;

/// "Sacrifice any number of lands. Search your library for up to that many land cards, put them onto the battlefield
/// tapped, then shuffle." The lands are chosen and sacrificed as the spell resolves, not as a cost (CR 701.21a, 608.2d);
/// choosing none is allowed. "That many" is the number actually sacrificed (CR 608.2c), and the search for up to that many
/// land cards (a stated quality, so fewer may be found, CR 701.23b) still happens, and the library is shuffled, when it is none.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let lands: Vec<ObjectId> = g.permanents_of(you).into_iter().filter(|&p| g.chars(p).is(CardType::Land)).collect();
    let chosen = choose::select(g, you, item.object, Purpose::Sacrifice, lands, select::land_cards(select::ANY_NUMBER))?;
    let sacrificed = g.sacrifice(you, &chosen).len();
    g.end_sentence();
    library::find_to_battlefield(g, you, item.object, select::land_cards(select::up_to(sacrificed as u32)), true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::SCAPESHIFT };
