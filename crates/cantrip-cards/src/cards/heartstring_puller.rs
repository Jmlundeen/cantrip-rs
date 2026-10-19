use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, create a 2/2 colorless Wizard Soldier creature token named Cadet."
fn cadet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::CADET, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        cadet,
        "When this creature enters, create a 2/2 colorless Wizard Soldier creature token named Cadet.",
    )],
    ..oracle::HEARTSTRING_PULLER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CADET];
