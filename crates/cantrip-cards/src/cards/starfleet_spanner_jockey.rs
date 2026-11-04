use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You've cast a noncreature spell this turn" (CR 601.2a): any noncreature spell this player cast this turn, of any
/// controller's effects and whether or not it resolved. A spell's types are kept with the cast (`Cast::types`).
fn cast_a_noncreature_spell_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history().casts_by(player).any(|c| !c.types.has(CardType::Creature))
}

/// "As long as you've cast a noncreature spell this turn, this creature can attack as though it didn't have defender"
/// (CR 702.3b, 609.4, 601.2a): only this creature, and only for a noncreature spell its own controller cast this turn -
/// nobody else's counts. It keeps defender for everything else.
fn may_attack_after_a_noncreature_spell(q: &dyn Query, asker: &Asker, creature: ObjectId) -> bool {
    creature == asker.source && cast_a_noncreature_spell_this_turn(q, asker.controller)
}

pub static CARD: CardDef = CardDef {
    attacks_despite_defender: &[may_attack_after_a_noncreature_spell],
    ..oracle::STARFLEET_SPANNER_JOCKEY
};
