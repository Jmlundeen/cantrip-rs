use cantrip_api::*;
use cantrip_mechanics::replacements::{plus_one_counters_applies, plus_one_counters_apply};
use cantrip_oracle as oracle;

/// "If one or more charge counters would be put on an artifact you control, that many plus one charge counters are put
/// on it instead." (CR 122.6, 614.1a.)
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    plus_one_counters_applies(q, asker, proposal, CounterKind::Charge, &[CardType::Artifact])
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    plus_one_counters_apply(proposal, CounterKind::Charge);
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(
        ReplacementKind::Counters,
        applies,
        apply,
        "puts one more charge counter",
    )],
    ..oracle::OPTIMIZATION_ANNEX
};
