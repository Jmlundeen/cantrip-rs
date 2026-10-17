use cantrip_api::*;
use cantrip_mechanics::replacements::{plus_one_counters_applies, plus_one_counters_apply};
use cantrip_oracle as oracle;

/// "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on it
/// instead" (CR 122.6, 614.1a): on a creature its controller controls, or one entering under them with counters (the
/// ruling), whoever puts them. Each Hardened Scales applies once (CR 614.5), so each adds one.
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    plus_one_counters_applies(q, asker, proposal, &[CardType::Creature])
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    plus_one_counters_apply(proposal);
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::Counters, applies, apply, "puts that many plus one +1/+1 counters instead")],
    ..oracle::HARDENED_SCALES
};
