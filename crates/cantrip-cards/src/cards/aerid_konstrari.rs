use cantrip_api::*;
use cantrip_mechanics::{artifacts_you_control, pump};
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_DIES;
use cantrip_oracle as oracle;

/// "Create a Heartwood token."
fn create_heartwood(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_heartwood(g, item.controller)
}

/// "{6}: Create a Heartwood token. Then Aerid Konstrari gets +X/+0 until end of turn, where X is the number of
/// artifacts you control": X is counted as this resolves, the new token included (CR 608.2c). If Aerid Konstrari has left
/// the battlefield, it gets nothing (CR 400.7).
fn make_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_heartwood(g, item.controller)?;
    let artifacts = artifacts_you_control(g, item.controller) as i16;
    pump(g, item.object, item.object, artifacts, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS_OR_DIES, create_heartwood, "When Aerid Konstrari enters or dies, create a Heartwood token."),
        ability(cost("{6}"), make_and_grow, "{6}: Create a Heartwood token. Then Aerid Konstrari gets +X/+0 until end of turn, where X is the number of artifacts you control."),
    ],
    ..oracle::AERID_KONSTRARI
};

pub static EXTRA: &[&CardDef] = &[&tokens::HEARTWOOD];
