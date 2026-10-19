use cantrip_api::*;
use cantrip_mechanics::{effects, prepare};
use cantrip_oracle as oracle;
/// Discard is a cost; drawing follows resolution (CR 602.2, 608.2).
pub static CARD: CardDef = CardDef {
 abilities: &[ability(TAP.and(discard(1, |_,_,_| true)), |g, i| effects::draw(g, i, 1), "{T}, Discard a card: Draw a card.")],
 enters: Some(prepare::enters_prepared),
 prepare: Some(&super::stingerquill_voxmancer::VICIOUS_VERSE),
 ..oracle::HALLWAY_HECKLER
};
