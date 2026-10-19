use cantrip_api::*;
use cantrip_mechanics::{effects,empower};
use cantrip_oracle as oracle;
/// Each counters-put event is one trigger, regardless of amount and planeswalker controller (CR 122.6a, 603.2).
fn counters(q:&dyn Query,me:&Asker,e:&Event)->Option<Fired>{
 if let Event::CountersPut{object,kind:CounterKind::Loyalty,amount,by,..}=e
 && *amount>0 && *by==me.controller && q.chars(*object).is(CardType::Planeswalker){Fired::PLAIN}else{None}
}
pub static CARD:CardDef=CardDef{abilities:&[
 triggered(when_asked(EventMask::COUNTERS,counters),effects::plus_one_counter_on_this,"Whenever you put one or more loyalty counters on a planeswalker, put a +1/+1 counter on this creature."),
 ability(cost("{6}"),|g,i|empower::jace(g,i.controller,i.object,2),"{6}: Empower Jace 2.")],..oracle::INSPIRED_TETHERMAGE};
