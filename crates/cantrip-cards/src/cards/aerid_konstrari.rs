use cantrip_api::*;
use cantrip_mechanics::pump;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{THIS_DIES, THIS_ENTERS};
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
    let artifacts = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&p| g.chars(p).is(CardType::Artifact))
        .count() as i16;
    pump(g, item.object, item.object, artifacts, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        // "Enters or dies" is one event or the other, never both at once, so it is written as the two triggers it is:
        // a leaves-the-battlefield trigger is looked back on (CR 603.10a), an enters trigger is not (CR 603.6a).
        triggered(THIS_ENTERS, create_heartwood, "When Aerid Konstrari enters, create a Heartwood token."),
        triggered(THIS_DIES, create_heartwood, "When Aerid Konstrari dies, create a Heartwood token.")
            .look_back(),
        ability(cost("{6}"), make_and_grow, "{6}: Create a Heartwood token. Then Aerid Konstrari gets +X/+0 until end of turn, where X is the number of artifacts you control."),
    ],
    ..oracle::AERID_KONSTRARI
};

pub static EXTRA: &[&CardDef] = &[&tokens::HEARTWOOD];
