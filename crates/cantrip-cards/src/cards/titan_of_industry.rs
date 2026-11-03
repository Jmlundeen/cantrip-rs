use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{counters, effects, targets, tokens};
use cantrip_oracle as oracle;

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
    mode(cantrip_mechanics::effects::destroy_first_target, "Destroy target artifact or enchantment.").targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    mode(effects::target_player_gains::<5>, "Target player gains 5 life.").targets(&[targets::PLAYER]),
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
