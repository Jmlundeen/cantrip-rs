use cantrip_api::*;
use cantrip_mechanics::{targets, damage_target};
use cantrip_oracle as oracle;
/// One simultaneous damage event to each qualifying opposing permanent (CR 120, 608.2c).
fn sweep(g: &mut dyn Game, i: &StackItem) -> Result<(), Suspend> {
 let victims: Vec<_> = g.state().battlefield().iter().copied().filter(|&id| g.controller(id) != i.controller && (g.chars(id).is(CardType::Creature) || g.chars(id).is(CardType::Planeswalker))).map(|id| (i.object, Target::Object(id), 1)).collect();
 g.damage_together(&victims); Ok(())
}
static MODAL: ModalDef = modes(&[
 mode(sweep, "Fulminous Forte deals 1 damage to each creature and planeswalker your opponents control."),
 mode(|g,i| { damage_target(g,i,0,5); Ok(()) }, "Fulminous Forte deals 5 damage to target creature or planeswalker.").targets(&[targets::CREATURE_OR_PLANESWALKER]),
]);
pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::FULMINOUS_FORTE };
