use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;
/// The restriction asks the recorded attack history, including creatures no longer attacking (CR 508.1).
fn attacked(q: &dyn Query, me: &Asker, t: Target) -> bool {
 (targets::CREATURE.filter)(q,me,t) && matches!(t, Target::Object(id) if q.history().attacked_this_turn(id))
}
/// Only a permanent with a prepare spell can become prepared (CR 722.3a); the verb owns that restriction.
fn prepare(g: &mut dyn Game, i: &StackItem) -> Result<(), Suspend> {
 if let Some(id) = targets::object(g,i,0) { g.become_prepared(id); } Ok(())
}
pub static CARD: CardDef = CardDef {
 abilities: &[
 mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
 ability(cost("{2}, {T}"), prepare, "{2}, {T}: Target creature that attacked this turn becomes prepared. Activate only as a sorcery.").targets(&[target(attacked)]).sorcery_speed(),
 ability(cost("{4}, {T}"), prepare, "{4}, {T}: Target creature becomes prepared.").targets(&[targets::CREATURE]),
 ], ..oracle::HEXHAVEN_DUELING_ARENA
};
