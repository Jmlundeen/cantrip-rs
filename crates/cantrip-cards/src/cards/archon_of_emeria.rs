use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Nonbasic lands your opponents control enter tapped." (CR 614.1c.) The land is judged as it would exist on the
/// battlefield (CR 614.12), under the player it enters under.
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let Proposal::ZoneChange(z) = proposal else { return false };
    z.to.is(ZoneKind::Battlefield) && z.controller != asker.controller && {
        let c = q.chars(z.object);
        c.is(CardType::Land) && !c.supertypes.basic
    }
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.tapped = true;
    }
}

/// "Each player can't cast more than one spell each turn." (CR 101.2.)
pub static CARD: CardDef = CardDef {
    cast_limit: 1,
    replacements: &[replacement(ReplacementKind::ZoneChange, applies, apply, "enters tapped")],
    ..oracle::ARCHON_OF_EMERIA
};
