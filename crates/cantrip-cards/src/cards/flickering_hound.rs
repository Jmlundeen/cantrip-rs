use cantrip_api::*;
use cantrip_mechanics::{targets,triggers,exile_and_return};
use cantrip_oracle as oracle;
/// Blink is a zone change: it returns as a new object and a token never returns (CR 400.7, 111.8).
fn blink(g: &mut dyn Game, i: &StackItem) -> Result<(), Suspend> {
 if let Some(id) = targets::object(g,i,0) { exile_and_return(g,&[id])?; } Ok(())
}
pub static CARD: CardDef = CardDef {
 abilities: &[triggered(triggers::CREATURE_SPELL_YOU_CAST,blink,"Whenever you cast a creature spell, exile up to one other target creature you control, then return that card to the battlefield under its owner's control.").targets(&[targets::OTHER_CREATURE_YOU_CONTROL.optional()])],
 ..oracle::FLICKERING_HOUND
};
