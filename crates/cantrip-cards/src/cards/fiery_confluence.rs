use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Fiery Confluence deals 1 damage to each creature.
fn creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let all: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    // One event (CR 120.2, 603.2c): "whenever a creature is dealt damage" sees them all at once.
    let hits: Vec<(ObjectId, Target, u16)> = all.into_iter().map(|id| (item.object, Target::Object(id), 1)).collect();
    g.damage_together(&hits);
    Ok(())
}

/// Fiery Confluence deals 2 damage to each opponent.
fn opponents(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(item.controller).map(|p| (item.object, Target::Player(p), 2)).collect();
    g.damage_together(&hits);
    Ok(())
}

/// Destroy target artifact.
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// Choose three; you may choose the same mode more than once (CR 700.2d).
static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef { text: "Fiery Confluence deals 1 damage to each creature.", targets: &[], effect: creatures, cost: ManaCost::ZERO },
        ModeDef { text: "Fiery Confluence deals 2 damage to each opponent.", targets: &[], effect: opponents, cost: ManaCost::ZERO },
        ModeDef {
            text: "Destroy target artifact.",
            targets: &[targets::ARTIFACT],
            effect: destroy_artifact,
            cost: ManaCost::ZERO,
        },
    ],
    min: 3,
    max: 3,
    escalate: None,
    repeat: true, allowed: None,
};

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::FIERY_CONFLUENCE };
