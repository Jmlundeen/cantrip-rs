use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Each player exiles all but the bottom six cards of their library face down" (CR 406.3, 101.4): every player's library
/// is cut at the same moment, so all the cards go together; a library of six or fewer cards loses none. The cards are
/// listed top first, so the bottom six are the last six.
fn exile_all_but_bottom_six(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let mut cards: Vec<ObjectId> = Vec::new();
    for player in g.state().apnap().collect::<Vec<_>>() {
        let library = g.state().zone(Zone::library(player));
        cards.extend_from_slice(&library[..library.len().saturating_sub(6)]);
    }
    g.exile_face_down(&cards, None);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS_IF_CAST,
            exile_all_but_bottom_six,
            "When this creature enters, if it was cast, each player exiles all but the bottom six cards of their library face down.",
        ),
        triggered(triggers::YOUR_UPKEEP, |g, item| effects::draw(g, item, 1), "At the beginning of your upkeep, draw a card."),
    ],
    ..oracle::DOOMSDAY_EXCRUCIATOR
};
