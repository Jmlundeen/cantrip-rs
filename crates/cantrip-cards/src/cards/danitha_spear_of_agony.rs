use cantrip_api::*;
use cantrip_mechanics::{effects,targets};
use cantrip_oracle as oracle;
/// One cast triggers once even when it targets several opposing creatures or players (CR 603.2c).
fn matching(q: &dyn Query, me: &Asker, e: &Event) -> Option<Fired> {
 if let Event::SpellCast { spell, player, .. } = e
 && *player == me.controller
 && let Some(item) = q.state().stack.iter().find(|i| i.kind == StackKind::Spell && i.object == *spell)
 && item.targets().iter().any(|t| match *t {
 Target::Player(p) => p != me.controller,
 Target::Object(id) => targets::permanent(q,id) && q.controller(id) != me.controller && q.chars(id).is(CardType::Creature),
 _ => false,
 }) { Fired::PLAIN } else { None }
}
pub static CARD: CardDef = CardDef {
 abilities: &[triggered(when_asked(EventMask::SPELL_CAST,matching), effects::plus_one_counter_on_this, "Whenever you cast a spell that targets an opponent or a creature an opponent controls, put a +1/+1 counter on Danitha.")],
 ..oracle::DANITHA_SPEAR_OF_AGONY
};
