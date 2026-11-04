use cantrip_api::*;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// The controller's lack of Forests creates a state trigger (CR 603.8).
fn no_forests(q: &dyn Query, asker: &Asker) -> bool {
    !lands::controls_land_type(q, asker.controller, q.words(asker.source).subtype(Subtype::Forest))
}

pub static CARD: CardDef = CardDef {
    abilities: &[state_triggered(no_forests, effects::sacrifice_self,
        "When you control no Forests, sacrifice this creature.")],
    attack_rules: &[lands::attack_rule_defender_controls::<{ lands::sub(Subtype::Forest) }>],
    ..oracle::GORILLA_PACK
};
