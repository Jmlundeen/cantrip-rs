use cantrip_api::*;
use cantrip_mechanics::{effects, history};
use cantrip_oracle as oracle;

/// "As long as you've scried or surveilled this turn, this creature can attack as though it didn't have defender"
/// (CR 702.3b, 609.4, 701.22d, 701.25d): only this creature, and only for a scry or surveil by its own controller this
/// turn, its ability's own included - nobody else's counts. It keeps defender for everything else.
fn may_attack_after_scrying_or_surveilling(q: &dyn Query, asker: &Asker, creature: ObjectId) -> bool {
    creature == asker.source && history::scried_or_surveilled_this_turn(q, asker.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}{U}"), |g, item| effects::surveil(g, item, 1), "{3}{U}: Surveil 1.")],
    attacks_despite_defender: &[may_attack_after_scrying_or_surveilling],
    ..oracle::SURVEILLANCE_PHANTASM
};
