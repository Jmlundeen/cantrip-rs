use cantrip_api::*;
use cantrip_oracle as oracle;

/// Metalcraft (CR 207.2c): "activate only if you control three or more artifacts" (CR 602.5b), counted through the
/// layers. A restriction on activating, not on having: the ability is on the Mox whatever the count.
fn metalcraft(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().filter(|&id| q.chars(id).is(CardType::Artifact)).count() >= 3
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "Metalcraft \u{2014} {T}: Add one mana of any color. Activate only if you control three or more artifacts.")
        .condition(metalcraft)],
    ..oracle::MOX_OPAL
};
