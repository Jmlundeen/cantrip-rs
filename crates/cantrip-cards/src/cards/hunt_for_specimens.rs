use cantrip_api::*;
use cantrip_mechanics::{learn, tokens};
use cantrip_oracle as oracle;

/// "Create a 1/1 black and green Pest creature token with 'When this token dies, you gain 1 life.' Learn." (CR 111.1, 701.48a); the
/// sentence ends before Learn (CR 608.2c).
fn learn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::PEST, 1)?;
    g.end_sentence();
    learn::learn(g, item)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], learn),
    ..oracle::HUNT_FOR_SPECIMENS
};

pub static EXTRA: &[&CardDef] = &[&tokens::PEST];
