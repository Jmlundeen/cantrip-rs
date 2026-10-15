use cantrip_api::*;
use cantrip_mechanics::{damage_target, discard_chosen, targets};
use cantrip_oracle as oracle;

/// Mode 0: return target creature card from your graveyard to your hand.
fn return_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

/// Mode 1: target player discards a card.
fn discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Player(player) = item.targets[0]
    {
        discard_chosen(g, player, item.object, 1)?;
    }
    Ok(())
}

/// Mode 2: destroy target artifact.
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// Mode 3: Kolaghan's Command deals 2 damage to any target.
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef {
            text: "Return target creature card from your graveyard to your hand.",
            targets: &[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD],
            effect: return_creature,
            cost: ManaCost::ZERO,
        },
        ModeDef { text: "Target player discards a card.", targets: &[targets::PLAYER], effect: discard, cost: ManaCost::ZERO },
        ModeDef {
            text: "Destroy target artifact.",
            targets: &[targets::ARTIFACT],
            effect: destroy_artifact,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "Kolaghan's Command deals 2 damage to any target.",
            targets: &[targets::ANY],
            effect: damage,
            cost: ManaCost::ZERO,
        },
    ],
    min: 2,
    max: 2,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::KOLAGHANS_COMMAND };
