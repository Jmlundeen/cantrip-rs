use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, surveil 1." (CR 701.42a): the top card is looked at and may be put into the graveyard.
fn surveil_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 1)
}

/// "Whenever a card leaves your graveyard, draw a card." (CR 603.6c, 603.10a): the card has left by the time this resolves,
/// so it is looked back at, and the card drawn is not itself the event.
fn draw_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOUR_UPKEEP, surveil_one, "At the beginning of your upkeep, surveil 1."),
        triggered(
            triggers::CARDS_LEAVE_YOUR_GRAVEYARD,
            draw_one,
            "Whenever a card leaves your graveyard, draw a card. This ability triggers only once each turn.",
        )
        .triggers_each_turn(1),
    ],
    ..oracle::VASH_FREELANCE_ARCHAEOLOGIST
};
