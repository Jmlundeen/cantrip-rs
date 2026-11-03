use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// Target a Wall, using the current type and the source's text (CR 115.1, 612.2).
fn wall(q: &dyn Query, asker: &Asker, target: Target) -> bool {
    targets::permanent_of_any_subtype(q, asker, target, &[Subtype::Wall])
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{R}"), effects::tap_targets, "{R}: Tap target Wall.")
        .targets(&[target(wall)])],
    ..oracle::ALI_BABA
};
