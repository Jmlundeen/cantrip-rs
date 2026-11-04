use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Return up to two target creature cards from your graveyard to your hand." (CR 115.1, 608.2b): none, one or two chosen; the
/// ones still legal return together.
fn return_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = targets::legal_objects(g, item);
    g.return_to_hand(&cards);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD.between(0, 2)], return_them),
    ..oracle::FIGHT_ON
};
