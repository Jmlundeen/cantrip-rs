use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_UPKEEP;
use cantrip_mechanics::{effects, history, tokens, your_turn};
use cantrip_oracle as oracle;

/// "Activate only during your turn and only if a card left your graveyard this turn" (CR 602.5): asked of the event history as
/// the ability is activated.
fn during_your_turn_after_a_card_left_your_graveyard(q: &dyn Query, asker: &Asker) -> bool {
    your_turn(q, asker) && history::a_card_left_your_graveyard_this_turn(q, asker)
}

/// "Create a 2/2 black Zombie Druid creature token" (CR 111.1).
fn create_zombie_druid(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ZOMBIE_DRUID, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOUR_UPKEEP, |g, item| effects::surveil(g, item, 1), "At the beginning of your upkeep, surveil 1. (Look at the top card of your library. You may put it into your graveyard.)"),
        ability(
            cost("{T}"),
            create_zombie_druid,
            "{T}: Create a 2/2 black Zombie Druid creature token. Activate only during your turn and only if a card left your graveyard this turn.",
        )
        .condition(during_your_turn_after_a_card_left_your_graveyard),
    ],
    ..oracle::ESSENCE_ANCHOR
};

/// The Zombie Druid token the ability makes.
pub static EXTRA: &[&CardDef] = &[&tokens::ZOMBIE_DRUID];
