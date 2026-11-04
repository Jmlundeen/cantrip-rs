use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever another artifact you control enters, create a 2/2 colorless Robot artifact creature token. This ability triggers only once
/// each turn" (CR 603.2, 603.4): the token is an artifact entering too, but the ability has triggered that turn already.
fn make_a_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ROBOT_2_2, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::ANOTHER_ARTIFACT_YOU_CONTROL_ENTERS,
        make_a_robot,
        "Whenever another artifact you control enters, create a 2/2 colorless Robot artifact creature token. This ability triggers only once each turn.",
    )
    .triggers_each_turn(1)],
    ..oracle::MECHAN_ASSEMBLER
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROBOT_2_2];
