use cantrip_api::*;
use cantrip_mechanics::{tokens,empower,typecycling};
use cantrip_oracle as oracle;
/// Three Cadets enter together before the empower action (CR 608.2c, 701.71).
fn effect(g:&mut dyn Game,i:&StackItem)->Result<(),Suspend>{tokens::create(g,i.controller,&tokens::CADET,3)?;empower::jace(g,i.controller,i.object,2)}
pub static CARD:CardDef=CardDef{spell:spell(&[],effect),abilities:&[typecycling::basic_landcycling(mana("{2}"),"Basic landcycling {2}")],..oracle::HEXHAVEN_BATTALION};
pub static EXTRA:&[&CardDef]=&[&tokens::CADET];
