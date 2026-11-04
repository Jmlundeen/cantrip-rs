use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target Spider" (CR 115.1, 205.3m): a permanent, any controller's, with the creature type Spider, as this creature's text
/// names it (CR 612.2).
fn spider(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_of_any_subtype(q, asker, t, &[Subtype::Spider])
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(cost("{2}{G}, {T}"), effects::plus_one_counter_on_target, "Web Support \u{2014} {2}{G}, {T}: Put a +1/+1 counter on target Spider. Activate only as a sorcery.")
            .targets(&[target(spider)])
            .sorcery_speed(),
    ],
    ..oracle::GUY_IN_THE_CHAIR
};
