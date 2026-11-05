use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Draw three cards. Creatures your opponents control get -3/-0 until end of turn." The creatures an opponent controls as the
/// spell resolves, after the cards are drawn (CR 611.2c): none that enters or comes under their control later is affected.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 3)?;
    effects::pump_creatures_your_opponents_control(g, item, -3, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::BEWILDERING_BLIZZARD };
