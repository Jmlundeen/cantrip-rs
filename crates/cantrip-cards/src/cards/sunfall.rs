use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

pub static EXTRA: &[&CardDef] = &[&tokens::INCUBATOR, &tokens::PHYREXIAN];

/// "Exile all creatures. Incubate X, where X is the number of creatures exiled this way." Every creature, whoever
/// controls it, goes in one move (CR 608.2f); X counts the moves made, those a replacement sent elsewhere included
/// (CR 608.2c). The Incubator is made even for X of zero (CR 701.53a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let exiled = g.exile_all(&creatures).len();
    tokens::incubate(g, item.controller, exiled as i16)
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ..oracle::SUNFALL
};
