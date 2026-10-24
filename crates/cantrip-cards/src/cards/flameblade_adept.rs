use cantrip_api::*;
use cantrip_mechanics::{pump, triggers};
use cantrip_oracle as oracle;

/// "this creature gets +1/+0 until end of turn." (CR 611.2)
fn pump_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, 0);
    Ok(())
}

/// "Whenever you cycle or discard a card": cycling discards the card (CR 702.29a), so its batch holds a discard event
/// and the one trigger fires once for a cycled card (CR 702.29d), as for any discard.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::YOU_DISCARD, pump_self, "Whenever you cycle or discard a card, this creature gets +1/+0 until end of turn.")],
    ..oracle::FLAMEBLADE_ADEPT
};
