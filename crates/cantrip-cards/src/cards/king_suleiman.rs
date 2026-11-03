use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{T}: Destroy target Djinn or Efreet." (CR 115.1).
fn destroy_target_djinn_efreet_target_0(
    q: &dyn Query,
    asker: &Asker,
    t: Target,
) -> bool {
    targets::permanent_of_any_subtype(q, asker, t, &[Subtype::Djinn, Subtype::Efreet])
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                TAP,
                effects::destroy_first_target,
                "{T}: Destroy target Djinn or Efreet.",
            )
            .targets(&[target(destroy_target_djinn_efreet_target_0)]),
    ],
    ..oracle::KING_SULEIMAN
};
