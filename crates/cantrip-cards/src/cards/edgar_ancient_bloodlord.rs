use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;
/// Look back to the other permanent's last controller and types (CR 603.10a).
fn dies(q:&dyn Query,me:&Asker,e:&Event)->Option<Fired>{
 if let Event::ZoneChange{old,from,to,..}=e
 && *old != me.source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard)
 && q.controller(*old)==me.controller && (q.chars(*old).is(CardType::Creature)||q.chars(*old).is(CardType::Planeswalker)) {Fired::PLAIN}else{None}
}
fn another(q:&dyn Query,me:&Asker,id:ObjectId)->bool{id!=me.source&&(q.chars(id).is(CardType::Creature)||q.chars(id).is(CardType::Planeswalker))}
/// The counter and menace apply even if the sacrificed permanent was Edgar's final companion (CR 608.2).
fn grow(g:&mut dyn Game,i:&StackItem)->Result<(),Suspend>{g.add_counters(i.object,CounterKind::PlusOne,1);g.add_effect(i.object,i.object,EffectKind::AddKeywords(Keywords::of(&[Keyword::Menace])),Duration::EndOfTurn);Ok(())}
pub static CARD:CardDef=CardDef{abilities:&[
 triggered(when_asked(EventMask::ZONE_CHANGE,dies),|g,i|effects::gain_life(g,i,1),"Whenever another creature or planeswalker you control dies, you gain 1 life.").look_back(),
 ability(cost("{2}").and(sacrifice(1,another)),grow,"{2}, Sacrifice another creature or planeswalker: Put a +1/+1 counter on Edgar. He gains menace until end of turn.")],..oracle::EDGAR_ANCIENT_BLOODLORD};
