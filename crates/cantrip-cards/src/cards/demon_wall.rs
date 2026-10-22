use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "As long as this creature has a counter on it, it can attack as though it didn't have defender" (CR 702.3b, 609.4): a
/// counter of any kind, asked of this creature alone. It keeps defender for everything else.
fn has_a_counter(q: &dyn Query, asker: &Asker, creature: ObjectId) -> bool {
    creature == asker.source && q.state().obj(creature).has_counters()
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{5}{B}"), effects::two_plus_one_counters_on_this, "{5}{B}: Put two +1/+1 counters on this creature.")],
    attacks_despite_defender: &[has_a_counter],
    ..oracle::DEMON_WALL
};
