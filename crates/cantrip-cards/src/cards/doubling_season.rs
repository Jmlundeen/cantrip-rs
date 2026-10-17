use cantrip_api::*;
use cantrip_mechanics::replacements::{double_counters_apply, DOUBLE_TOKENS};
use cantrip_oracle as oracle;

/// "If an effect would put one or more counters on a permanent you control" (CR 122.6, 614.1a): counters of any kind put by
/// an effect - a resolving spell's or ability's, or as the permanent enters with them, under its controller as it would
/// exist then (the rulings: a planeswalker's loyalty, a battle's defense) - but not as a cost (a loyalty ability's, the
/// ruling) nor by a turn-based action (`CounterPlacement::effect`). Whoever puts them, on a permanent its controller
/// controls.
fn effect_puts_counters_on_your_permanent(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let Proposal::Counters(c) = proposal else { return false };
    c.effect && c.controller == asker.controller && (c.entering || q.is_on_battlefield(c.object)) && c.counts.iter().any(|&n| n > 0)
}

/// "... it puts twice that many of those counters on that permanent instead": each kind doubled; two Doubling Seasons make
/// four times as many (the ruling, CR 614.5).
fn twice_that_many(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    double_counters_apply(proposal);
}

pub static CARD: CardDef = CardDef {
    // "If an effect would create one or more tokens under your control, it creates twice that many of those tokens instead"
    // (CR 111.1, 614.1a): every token is made by an effect.
    replacements: &[
        DOUBLE_TOKENS,
        replacement(ReplacementKind::Counters, effect_puts_counters_on_your_permanent, twice_that_many, "puts twice that many counters instead"),
    ],
    ..oracle::DOUBLING_SEASON
};
