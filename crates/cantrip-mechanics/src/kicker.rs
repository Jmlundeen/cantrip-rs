//! Kicker and multikicker (CR 702.33): an optional additional cost, paid once or any number of times. The cost and the
//! count of payments (`Obj::kicked`) are the core's (`CardDef::kicker`); what a card does with them is its own, except
//! "enters with a counter for each time it was kicked" (CR 702.33d, 614.1c), which Batroc, the Leaper (+1/+1) and
//! Everflowing Chalice (charge) print.
//!
//! A card with it gives `enters_with_counters::<KIND>` as its `enters`, `KIND` being `CounterKind::X as usize`.

use cantrip_api::*;

/// Enters with a counter of kind `KIND` for each time it was kicked. An object put onto the battlefield without being
/// cast was never kicked and enters bare.
pub fn enters_with_counters<const KIND: usize>(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[KIND] += g.obj(entering.object).kicked as i16;
    Ok(())
}
