use cantrip_api::*;
use cantrip_mechanics::{creature_cards_in_graveyard, mobilize};
use cantrip_oracle as oracle;

/// "Mobilize X, where X is the number of creature cards in your graveyard." (CR 702.181a, 107.3, 608.2h): X is counted once, as the
/// triggered ability resolves, in its controller's graveyard only; tokens are not cards.
fn mobilize_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = creature_cards_in_graveyard(g, item.controller);
    mobilize::mobilize(g, item, x)
}

pub static CARD: CardDef = CardDef {
    abilities: &[mobilize::trigger_x(
        mobilize_x,
        "Mobilize X, where X is the number of creature cards in your graveyard. (Whenever this creature attacks, create X tapped and attacking 1/1 red Warrior creature tokens. Sacrifice them at the beginning of the next end step.)",
    )],
    delayed: &[mobilize::SACRIFICE],
    ..oracle::AVENGER_OF_THE_FALLEN
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
