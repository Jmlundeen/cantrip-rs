use cantrip_api::*;
use cantrip_mechanics::{scry, tokens, triggers};
use cantrip_oracle as oracle;

/// "Scry 1 and create a Treasure token." (CR 701.22, 111.10a.) The scry is done first, as the text orders them.
fn scry_and_treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 1)?;
    tokens::create(g, item.controller, &tokens::TREASURE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // A leaves-the-battlefield trigger is looked back on (CR 603.10a).
    abilities: &[triggered(triggers::THIS_DIES, scry_and_treasure, "When this creature dies, scry 1 and create a Treasure token.")],
    ..oracle::GREEDY_FREEBOOTER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
