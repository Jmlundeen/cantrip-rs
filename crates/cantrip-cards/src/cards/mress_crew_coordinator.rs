use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Landfall — Whenever a land you control enters, create two 1/1 red Officer creature tokens." The tokens are created
/// together (CR 111.1, 111.4).
fn two_officers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OFFICER, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::LANDFALL,
        two_officers,
        "Landfall — Whenever a land you control enters, create two 1/1 red Officer creature tokens.",
    )],
    ..oracle::MRESS_CREW_COORDINATOR
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
