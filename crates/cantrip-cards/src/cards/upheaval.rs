use cantrip_api::*;
use cantrip_oracle as oracle;

/// Return all permanents to their owners' hands, together (CR 701.19 is not involved; a token that reaches a hand
/// ceases to exist, CR 704.5d).
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let permanents = g.state().battlefield().to_vec();
    g.return_to_hand(&permanents);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::UPHEAVAL
};
