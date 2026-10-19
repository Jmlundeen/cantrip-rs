use cantrip_api::*;
use cantrip_mechanics::{targets,fight};
use cantrip_oracle as oracle;
/// Red or white opposing creature or planeswalker; both targeting restrictions remain checked on resolution (CR 608.2b).
fn opposing(q:&dyn Query,me:&Asker,t:Target)->bool{
 (targets::CREATURE_OR_PLANESWALKER.filter)(q,me,t)&&matches!(t,Target::Object(id) if q.controller(id)!=me.controller && (q.chars(id).colors.has(Color::Red)||q.chars(id).colors.has(Color::White)))
}
fn effect(g:&mut dyn Game,i:&StackItem)->Result<(),Suspend>{
 let victim=targets::object(g,i,0);
 if let Some(id)=victim{g.add_effect(i.object,id,EffectKind::LoseAllAbilities,Duration::EndOfTurn);}
 if let (Some(a),Some(b))=(targets::object(g,i,1),victim){fight::bite(g,a,b);}Ok(())
}
pub static CARD:CardDef=CardDef{spell:spell(&[target(opposing),targets::CREATURE_YOU_CONTROL],effect),..oracle::FLOURISHING_GRAPPLE};
