use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This creature can't attack unless you control another artifact" (CR 508.1c): asked of this creature as attackers are declared; the
/// other artifacts are the permanents its controller controls that are artifacts now, this creature not among them. Once declared it
/// stays attacking whatever happens to them (CR 506.4).
fn attacks_only_with_another_artifact(q: &dyn Query, asker: &Asker, attacker: ObjectId, _: Target) -> bool {
    if asker.source != attacker {
        return true;
    }
    let me = q.controller(asker.source);
    q.state().battlefield().iter().any(|&id| id != asker.source && q.controller(id) == me && q.chars(id).is(CardType::Artifact))
}

pub static CARD: CardDef = CardDef { attack_rules: &[attacks_only_with_another_artifact], ..oracle::MOUSER_MARK_III };
