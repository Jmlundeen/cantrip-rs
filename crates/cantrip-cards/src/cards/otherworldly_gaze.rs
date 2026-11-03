use cantrip_api::*;
use cantrip_mechanics::{effects, flashback};
use cantrip_oracle as oracle;

/// "Surveil 3. (Look at the top three cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.)" (CR 701.25a).
fn surveil_3(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 3)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], surveil_3),
    ways: &[flashback::way(cost("{1}{U}"), "Flashback {1}{U}")],
    ..oracle::OTHERWORLDLY_GAZE
};
