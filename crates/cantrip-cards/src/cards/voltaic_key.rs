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
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{1}, {T}: Untap target artifact.",
        cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
        targets: &[targets::ARTIFACT],
        effect: untap_artifact,
        ..ActivatedDef::BLANK
    })],
    ..oracle::VOLTAIC_KEY
};
