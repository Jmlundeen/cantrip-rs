use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "This creature can't be blocked by creatures with power 2 or greater as long as defending player controls a snow land." (CR 509.1b):
/// whether `blocker` may block, false for a blocker of power 2 or more while the defending player controls a snow land.
fn may_block(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    attacker != source
        || !q.defending_player(attacker).is_some_and(|defending| lands::controls_snow_land(q, defending))
        || q.chars(blocker).power.is_none_or(|power| power < 2)
}

pub static CARD: CardDef = CardDef {
    block_rules: &[may_block],
    ..oracle::ARCTIC_FOXES
};
