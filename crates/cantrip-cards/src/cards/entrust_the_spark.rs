use cantrip_api::*;
use cantrip_mechanics::{choose,library};
use cantrip_oracle as oracle;
/// Optional sacrifice happens on resolution, before the conditional search (CR 608.2d, 701.21).
fn effect(g:&mut dyn Game,i:&StackItem)->Result<(),Suspend>{
 let eligible=g.permanents_of(i.controller).into_iter().filter(|&id|g.chars(id).is(CardType::Planeswalker)).collect();
 let chosen=choose::cards(g,i.controller,i.object,Purpose::Sacrifice,eligible,1,true)?;
 if !chosen.is_empty(){g.sacrifice(i.controller,&chosen);library::search_to_battlefield(g,i.controller,i.object,library::Find::ONE,false,|q,id|q.chars(id).is(CardType::Planeswalker))?;}Ok(())
}
pub static CARD:CardDef=CardDef{spell:spell(&[],effect),..oracle::ENTRUST_THE_SPARK};
