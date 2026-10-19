use cantrip_api::*;
use cantrip_mechanics::{targets,triggers};
use cantrip_oracle as oracle;
fn opposing(q:&dyn Query,_:&Asker,t:Target)->bool{matches!(t,Target::Object(id) if targets::controlled_by_slot_opponent(q,id)&&q.chars(id).is(CardType::Creature))}
/// X is read as this resolves, independently of how many creatures were targeted (CR 608.2h).
fn effect(g:&mut dyn Game,i:&StackItem)->Result<(),Suspend>{
 let n=g.state().zone(Zone::graveyard(i.controller)).iter().map(|&id|g.mana_value(id)).max().unwrap_or(0).min(i16::MAX as u32) as i16;
 for id in targets::legal_objects(g,i){g.add_counters(id,CounterKind::MinusOne,n);}Ok(())
}
pub static CARD:CardDef=CardDef{abilities:&[triggered(triggers::THIS_ENTERS,effect,"When Hapatra enters, for each opponent, put X -1/-1 counters on up to one target creature that player controls, where X is the greatest mana value among cards in your graveyard.").targets(&[target(opposing).between(0,TargetSpec::EACH_OPPONENT)])],..oracle::HAPATRA_THE_DESERT_FANG};
