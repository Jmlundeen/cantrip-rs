use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// What the trigger does depends on how many times this ability of this Belladonna has resolved this turn before
/// (CR 608.2m): this turn's history counts them. First, gain 1 life. Second, draw a card. Third, a +1/+1 counter on each
/// creature you control. A fourth and later resolution does nothing.
fn token_enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    match g.history().resolutions(item.object, item.ability) {
        0 => effects::gain_life(g, item, 1)?,
        1 => effects::draw(g, item, 1)?,
        2 => {
            effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
        }
        _ => {}
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, triggers::token_you_control_enters), token_enters, "Whenever a token you control enters, you gain 1 life if this is the first time this ability has resolved this turn. If it's the second time, draw a card. If it's the third time, put a +1/+1 counter on each creature you control.")],
    ..oracle::BELLADONNA_TOOK
};
