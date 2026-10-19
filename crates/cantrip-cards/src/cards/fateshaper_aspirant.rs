use cantrip_api::*;
use cantrip_mechanics::{targets,triggers,effects};
use cantrip_oracle as oracle;
/// Legendary card in your graveyard; card types are unrestricted (CR 115.1).
fn legendary(q:&dyn Query, me:&Asker,t:Target)->bool {
 (targets::CARD_IN_YOUR_GRAVEYARD.filter)(q,me,t) && matches!(t,Target::Object(id) if q.chars(id).supertypes.legendary)
}
/// A counter remains after vigilance and indestructible wear off (CR 122, 611.2a).
fn protect(g:&mut dyn Game,i:&StackItem)->Result<(),Suspend>{
 if let Some(id)=targets::object(g,i,0){g.add_counters(id,CounterKind::PlusOne,1);g.add_effect(i.object,id,EffectKind::AddKeywords(Keywords::of(&[Keyword::Vigilance,Keyword::Indestructible])),Duration::EndOfTurn);}Ok(())
}
static MODAL: ModalDef = modes(&[
 mode(effects::return_first_target_to_hand,"Return target legendary card from your graveyard to your hand.").targets(&[target(legendary)]),
 mode(protect,"Put a +1/+1 counter on target creature. It gains vigilance and indestructible until end of turn.").targets(&[targets::CREATURE]),
]);
pub static CARD:CardDef=CardDef{abilities:&[modal_triggered(triggers::THIS_ENTERS,&MODAL,"When this creature enters, choose one — Return target legendary card from your graveyard to your hand; or put a +1/+1 counter on target creature. It gains vigilance and indestructible until end of turn.")],..oracle::FATESHAPER_ASPIRANT};
