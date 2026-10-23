use cantrip_api::*;
use cantrip_mechanics::typecycling;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Creatures you control get +1/+0 and gain first strike until end of turn: the creatures there are as it resolves
/// (CR 611.2c).
fn pump_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        for id in effects::pump_creatures_you_control(g, item, 1, 0) {
            effects::gain_until_end_of_turn(g, item.object, id, Keyword::FirstStrike);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, pump_creatures, "When Eagles of the North enters, creatures you control get +1/+0 and gain first strike until end of turn."),
        typecycling::ability(Subtype::Plains, mana("{1}"), "Plainscycling {1} ({1}, Discard this card: Search your library for a Plains card, reveal it, put it into your hand, then shuffle.)"),
    ],
    ..oracle::EAGLES_OF_THE_NORTH
};
