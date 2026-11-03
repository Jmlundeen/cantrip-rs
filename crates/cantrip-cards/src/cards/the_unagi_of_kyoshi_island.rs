use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        // "Ward—Waterbend {4}": the opponent who targets it pays {4} by tapping untapped artifacts and creatures, or with mana, or it is countered
        // (CR 702.21a, 701.67b).
        triggers::ward_waterbend::<4>("Ward—Waterbend {4} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {4}. They can tap their artifacts and creatures to help. Each one pays for {1}.)"),
        triggered(triggers::AN_OPPONENT_DRAWS_THEIR_SECOND_CARD, |g, item| effects::draw(g, item, 2), "Whenever an opponent draws their second card each turn, you draw two cards."),
    ],
    ..oracle::THE_UNAGI_OF_KYOSHI_ISLAND
};
