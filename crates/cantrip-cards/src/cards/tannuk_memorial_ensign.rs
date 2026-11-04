use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Tannuk deals 1 damage to each opponent. If this is the second time this ability has resolved this turn, draw a card." Each
/// resolution follows its instructions in order (CR 608.2c): the damage every time, the card only on the second.
fn ping_and_draw_the_second_time(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 1);
    if g.history().resolutions(item.object, item.ability) == 1 {
        effects::draw(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::LANDFALL,
        ping_and_draw_the_second_time,
        "Landfall \u{2014} Whenever a land you control enters, Tannuk deals 1 damage to each opponent. If this is the second time this ability has resolved this turn, draw a card.",
    )],
    ..oracle::TANNUK_MEMORIAL_ENSIGN
};
