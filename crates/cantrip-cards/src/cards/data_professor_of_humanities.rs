use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::seek_out::seek_out;
use cantrip_oracle as oracle;

/// "Seek out — Whenever you draw your second card each turn, create two 1/1 blue Faerie creature tokens with flying."
/// The trigger fires once a turn, on the second card drawn, whoever's turn it is (CR 603.2, 121.2), and both tokens
/// enter together as one event (CR 111.1, 603.6a). "Seek out" is an ability word the Comp Rules give no meaning (CR 207.2c).
fn create_two_faeries(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_by_rule(g, item.controller, &tokens::FAERIE, 2, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[seek_out(
        create_two_faeries,
        &[],
        "Seek out \u{2014} Whenever you draw your second card each turn, create two 1/1 blue Faerie creature tokens with flying.",
    )],
    ..oracle::DATA_PROFESSOR_OF_HUMANITIES
};

pub static EXTRA: &[&CardDef] = &[&tokens::FAERIE];
