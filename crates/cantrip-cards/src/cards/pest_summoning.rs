use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Create two 1/1 black and green Pest creature tokens with "When this token dies, you gain 1 life."" (CR 111.1).
fn create_two_1_1_black(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::PEST, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], create_two_1_1_black),
    ..oracle::PEST_SUMMONING
};

pub static EXTRA: &[&CardDef] = &[&tokens::PEST];
