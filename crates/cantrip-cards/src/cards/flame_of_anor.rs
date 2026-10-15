use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// Mode 0: target player draws two cards.
fn draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Player(player) = item.targets[0]
    {
        g.draw(player, 2);
    }
    Ok(())
}

/// Mode 1: destroy target artifact.
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// Mode 2: Flame of Anor deals 5 damage to target creature.
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

/// "If you control a Wizard as you cast this spell, you may choose two instead" (CR 700.2): asked once, as the modes
/// are announced, so a Wizard that leaves in response leaves both modes chosen.
fn two_with_a_wizard(q: &dyn Query, asker: &Asker) -> u8 {
    if q.permanents_of(asker.controller).into_iter().any(|id| q.chars(id).has_subtype(Subtype::Wizard)) { 2 } else { 1 }
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef { text: "Target player draws two cards.", targets: &[targets::PLAYER], effect: draw_two, cost: ManaCost::ZERO },
        ModeDef {
            text: "Destroy target artifact.",
            targets: &[targets::ARTIFACT],
            effect: destroy_artifact,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "Flame of Anor deals 5 damage to target creature.",
            targets: &[targets::CREATURE],
            effect: damage,
            cost: ManaCost::ZERO,
        },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef =
    CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), modes_up_to: Some(two_with_a_wizard), ..oracle::FLAME_OF_ANOR };
