use cantrip_api::*;
use cantrip_mechanics::artifacts_you_control;
use cantrip_oracle as oracle;

/// Metalcraft (CR 207.2c): "activate only if you control three or more artifacts" (CR 602.5b), counted through the
/// layers. A restriction on activating, not on having: the ability is on the Mox whatever the count.
fn metalcraft(q: &dyn Query, asker: &Asker) -> bool {
    artifacts_you_control(q, asker.controller) >= 3
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "Metalcraft \u{2014} {T}: Add one mana of any color. Activate only if you control three or more artifacts.")
        .condition(metalcraft)],
    ..oracle::MOX_OPAL
};
