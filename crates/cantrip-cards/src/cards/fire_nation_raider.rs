use cantrip_api::*;
use cantrip_mechanics::{raid, tokens};
use cantrip_oracle as oracle;

/// "Create a Clue token": created, which is not investigating (CR 701.16a; the card's ruling) -- nothing here triggers on
/// investigating.
fn create_a_clue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[raid::enters(create_a_clue, "Raid \u{2014} When this creature enters, if you attacked this turn, create a Clue token.")],
    ..oracle::FIRE_NATION_RAIDER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
