use cantrip_api::*;
use cantrip_mechanics::replacements::plus_one_counters_applies;
use cantrip_oracle as oracle;

/// "If one or more +1/+1 counters would be put on a creature you control" (CR 122.6, 614.1a): on a creature its controller
/// controls, or one entering under them with counters (the ruling), whoever puts them.
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    plus_one_counters_applies(q, asker, proposal, &[CardType::Creature])
}

/// "... twice that many +1/+1 counters are put on it instead": the +1/+1 counters alone; counters of other kinds put with
/// them are left as they are. Two Menaces make four times as many (the ruling, CR 614.5).
fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::Counters(c) = proposal {
        c.counts[CounterKind::PlusOne as usize] *= 2;
    }
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::Counters, applies, apply, "puts twice that many +1/+1 counters instead")],
    ..oracle::CORPSEJACK_MENACE
};
