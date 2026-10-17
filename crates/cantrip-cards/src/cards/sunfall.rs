use cantrip_api::*;
use cantrip_mechanics::{effects, tokens};
use cantrip_oracle as oracle;

pub static EXTRA: &[&CardDef] = &[&tokens::INCUBATOR, &tokens::PHYREXIAN];

/// "Exile all creatures. Incubate X, where X is the number of creatures exiled this way." Every creature, whoever
/// controls it, goes in one move (CR 608.2f); X counts the moves made, those a replacement sent elsewhere included
/// (CR 608.2c). The Incubator is made even for X of zero (CR 701.53a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled = effects::exile_all_creatures(g);
    tokens::incubate(g, item.controller, exiled as i16)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::SUNFALL
};
