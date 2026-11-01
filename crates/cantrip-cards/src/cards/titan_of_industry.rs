use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{counters, targets, tokens};
use cantrip_oracle as oracle;

/// Mode 1: "Destroy target artifact or enchantment" (CR 701.8, 608.2b).
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// Mode 2: "Target player gains 5 life" (CR 119.3), nothing if the player is no longer a legal target (CR 608.2b).
fn gain_five(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        g.gain_life(player, 5);
    }
    Ok(())
}

/// Mode 3: "Create a 4/4 green Rhino Warrior creature token" (CR 111.1), under the ability's controller.
fn rhino(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::RHINO_WARRIOR, 1)?;
    Ok(())
}

/// Mode 4: "Put a shield counter on a creature you control" (CR 122.1c): chosen as the ability resolves, not targeted
/// (CR 608.2d), so it can be the Rhino the third mode just made (the modes are followed in the order written, CR 608.2c). A lone creature
/// is taken without asking; none at all does nothing.
fn shield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counters::put_on_creature_you_control(g, item, CounterKind::Shield, 1)
}

/// "Choose two" (CR 700.2, 700.2b): the modes are carried out in the order written (CR 608.2c). Only the first two have
/// targets (CR 700.2c).
static MODAL: ModalDef = modes(&[
    mode(destroy, "Destroy target artifact or enchantment.").targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    mode(gain_five, "Target player gains 5 life.").targets(&[targets::PLAYER]),
    mode(rhino, "Create a 4/4 green Rhino Warrior creature token."),
    mode(shield, "Put a shield counter on a creature you control."),
])
.choose(2, 2);

pub static EXTRA: &[&CardDef] = &[&tokens::RHINO_WARRIOR];

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When this creature enters, choose two \u{2014} Destroy target artifact or enchantment; target player gains 5 life; create a 4/4 green Rhino Warrior creature token; or put a shield counter on a creature you control.",
    )],
    ..oracle::TITAN_OF_INDUSTRY
};
