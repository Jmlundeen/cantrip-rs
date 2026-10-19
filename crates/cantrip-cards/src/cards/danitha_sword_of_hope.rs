use cantrip_api::*;
use cantrip_mechanics::{effects,targets};
use cantrip_oracle as oracle;
/// Equipment and friendly-target clauses describe one trigger, limited to once per turn (CR 603.2c,h).
fn matching(q: &dyn Query, me: &Asker, e: &Event) -> Option<Fired> {
 if let Event::SpellCast { spell, player, .. } = e
 && *player == me.controller
 && (q.chars(*spell).has_subtype(Subtype::Equipment) || q.state().stack.iter().find(|i| i.kind == StackKind::Spell && i.object == *spell).is_some_and(|i|
 i.targets().iter().filter_map(|t| t.object()).any(|id| targets::permanent(q,id) && q.controller(id) == me.controller && q.chars(id).is(CardType::Creature)))) {
 Fired::PLAIN
 } else { None }
}
pub static CARD: CardDef = CardDef {
 abilities: &[triggered(when_asked(EventMask::SPELL_CAST,matching), |g,i| effects::draw(g,i,1), "Whenever you cast an Equipment spell or a spell that targets a creature you control, draw a card. This ability triggers only once each turn.").triggers_each_turn(1)],
 ..oracle::DANITHA_SWORD_OF_HOPE
};
