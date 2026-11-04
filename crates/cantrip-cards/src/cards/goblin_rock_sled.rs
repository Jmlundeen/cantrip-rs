use cantrip_api::*;
use cantrip_mechanics::{history, lands};
use cantrip_oracle as oracle;

/// "This creature doesn't untap during your untap step if it attacked during your last turn." (CR 502.3, 604.1).
fn doesnt_untap_if_attacked_last_turn(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id == asker.source && history::attacked_during_last_turn_of(q, asker.controller, asker.source)
}

pub static CARD: CardDef = CardDef {
    attack_rules: &[lands::attack_rule_defender_controls::<{ lands::sub(Subtype::Mountain) }>],
    untap_bans: &[doesnt_untap_if_attacked_last_turn],
    ..oracle::GOBLIN_ROCK_SLED
};
