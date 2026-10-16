use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Return all artifacts target player owns to their hand (CR 108.3): every artifact on the battlefield whose owner is the
/// target player, whoever controls it now, in one simultaneous move. The set is taken as the spell resolves.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(owner) = item.targets[0].player() else { return Ok(()) };
    let artifacts: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.obj(id).owner == owner && g.chars(id).is(CardType::Artifact))
        .collect();
    g.return_to_hand(&artifacts);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::HURKYLS_RECALL
};
