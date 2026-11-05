use cantrip_api::*;
use cantrip_mechanics::{effects, lands, triggers};
use cantrip_oracle as oracle;

/// "Search your library for a basic land card, reveal it, put it into your hand, then shuffle" (CR 701.23): the card may be
/// left unfound (CR 701.23b), and the library is shuffled either way.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::search_basic_land_to_hand(g, item.controller, item.object)
}

/// The Omen (CR 720): cast from the hand as an Omen sorcery for {G}; as it resolves it is shuffled into its owner's library
/// (CR 720.3d, the core's).
pub static ROOST_SEEK: CardDef = CardDef { spell: spell(&[], search), ..oracle::ROOST_SEEK };

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, effects::gain_three_life, "When this creature enters, you gain 3 life.")],
    omen: Some(&ROOST_SEEK),
    ..oracle::SAGU_WILDLING
};

pub static EXTRA: &[&CardDef] = &[&ROOST_SEEK];
