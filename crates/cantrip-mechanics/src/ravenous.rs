//! Ravenous (CR 702.156a): "This creature enters with X +1/+1 counters on it. If X is 5 or more, draw a card when it
//! enters." Jacked Rabbit and Mawloc print it.
//!
//! A card with ravenous lists `enters` as its `enters` and `draw_trigger(text)` among its `abilities`.

use crate::triggers::THIS_ENTERS;
use cantrip_api::*;

/// "This creature enters with X +1/+1 counters on it", for the X it was cast with. Put onto the battlefield without
/// being cast, X is 0.
pub fn enters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::PlusOne as usize] += entering.x as i16;
    Ok(())
}

/// "If X is 5 or more" (CR 603.4): checked as it enters and again as the trigger resolves.
fn x_is_five_or_more(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).x >= 5
}

fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// "When this creature enters, if X is 5 or more, draw a card." `text` names the card.
pub const fn draw_trigger(text: &'static str) -> AbilityDef {
    triggered(THIS_ENTERS, draw, text)
        .condition(x_is_five_or_more)
}
