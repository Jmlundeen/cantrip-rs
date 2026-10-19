use cantrip_api::*;
use cantrip_oracle as oracle;
/// The replacement changes only artifact tokens created under your control.
fn applies(q:&dyn Query,me:&Asker,p:&Proposal)->bool{matches!(p,Proposal::TokenCreation(t) if t.controller==me.controller && q.token_chars(t.token, t.copy).is(CardType::Artifact))}
fn replace(q:&dyn Query,_:&Asker,p:&mut Proposal){if let Proposal::TokenCreation(t)=p{t.token=q.card_id(&DRAGON);t.copy=None;}}
static DRAGON:CardDef=CardDef{name:"Dragon Token",has_cost:false,types:TypeSet::of(&[CardType::Creature]),subtypes:SubtypeSet::of(&[Subtype::Dragon]),colors:Some(ColorSet::of(Color::Red)),power:Some(5),toughness:Some(5),keywords:Keywords::of(&[Keyword::Flying]),..CardDef::BLANK};
pub static CARD:CardDef=CardDef{replacements:&[replacement(ReplacementKind::TokenCreation,applies,replace,"creates 5/5 red Dragon creature tokens with flying instead")],..oracle::DRACONIC_VISITOR};
pub static EXTRA:&[&CardDef]=&[&DRAGON];
