use cantrip_api::*;
use cantrip_mechanics::{library, triggers, ANY_NUMBER};
use cantrip_oracle as oracle;

fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    library::search_to_hand(g, player, item.object, library::Find::ONE.up_to(ANY_NUMBER), |q, c| {
        q.chars(c).name == "Andromedan Automatons"
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, search, "When this creature enters, you may search your library for any number of cards named Andromedan Automatons, reveal them, put them into your hand, then shuffle.")
    ],
    ..oracle::ANDROMEDAN_AUTOMATONS
};
