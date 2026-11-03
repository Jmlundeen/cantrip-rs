use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever you sacrifice another permanent" (CR 701.21a, 603.10a): a permanent of yours that moved from the battlefield by a
/// sacrifice, other than the Peddlers itself. Several permanents sacrificed together each trigger it.
fn you_sacrifice_another_permanent(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let sacrificed = triggers::sacrificed_by(q, me.controller, event)?;
    if sacrificed == me.source { None } else { Fired::PLAIN }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, you_sacrifice_another_permanent),
        effects::plus_one_counter_on_this,
        "Whenever you sacrifice another permanent, put a +1/+1 counter on this creature.",
    )
    .on_sacrifice()],
    ..oracle::PIRATE_PEDDLERS
};
