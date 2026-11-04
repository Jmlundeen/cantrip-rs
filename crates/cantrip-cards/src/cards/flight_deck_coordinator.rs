use cantrip_api::*;
use cantrip_mechanics::effects::gain_life;
use cantrip_mechanics::triggers::{YOUR_END_STEP, you_control_two_or_more_tapped_creatures};
use cantrip_oracle as oracle;

/// "You gain 2 life" (CR 119.3).
fn gain_two_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        gain_two_life,
        "At the beginning of your end step, if you control two or more tapped creatures, you gain 2 life.",
    )
    .condition(you_control_two_or_more_tapped_creatures)],
    ..oracle::FLIGHT_DECK_COORDINATOR
};
