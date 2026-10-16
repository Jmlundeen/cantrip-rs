use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If a nontoken creature would enter and it wasn't cast, exile it instead." (CR 614.1a.) It belongs to another
/// permanent, so it functions only while it is on the battlefield (CR 604.1), and it judges the creature as it would
/// exist there (CR 614.12). The only way a permanent reaches the battlefield from the stack is a spell resolving, so
/// `ZoneReason::Resolve` is exactly "it was cast".
fn applies(q: &dyn Query, _: &Asker, proposal: &Proposal) -> bool {
    let Proposal::ZoneChange(z) = proposal else { return false };
    z.to.is(ZoneKind::Battlefield) && z.reason != ZoneReason::Resolve && !q.obj(z.object).token && q.chars(z.object).is(CardType::Creature)
}

fn apply(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.to = Zone::exile(q.obj(z.object).owner);
    }
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::ZoneChange, applies, apply, "exiles a creature that wasn't cast")],
    ..oracle::CONTAINMENT_PRIEST
};
