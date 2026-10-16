use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Artifacts and creatures your opponents control enter tapped." (CR 614.1c.) The permanent is judged as it would
/// exist on the battlefield (CR 614.12), under the player it enters under.
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let Proposal::ZoneChange(z) = proposal else { return false };
    z.to.is(ZoneKind::Battlefield) && !z.tapped && z.controller != asker.controller && {
        let c = q.chars(z.object);
        c.is(CardType::Artifact) || c.is(CardType::Creature)
    }
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.tapped = true;
    }
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::ZoneChange, applies, apply, "your opponents' artifacts and creatures enter tapped")],
    ..oracle::SPIDER_WOMAN_STUNNING_SAVIOR
};
