use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Untap target artifact, if it is still a legal target.
fn untap_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(artifact) = targets::object(g, item, 0) {
        g.untap(artifact);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}, {T}"), untap_artifact, "{1}, {T}: Untap target artifact.")
        .targets(&[targets::ARTIFACT])],
    ..oracle::VOLTAIC_KEY
};
