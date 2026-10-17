use cantrip_api::*;
use cantrip_mechanics::{lands, lands_you_control};
use cantrip_oracle as oracle;

/// "Activate only if you control five or more lands" (CR 602.5b): checked as the ability is activated, before its cost
/// is paid, so Cryptic Caves counts as one of the five and its own sacrifice does not undo the activation.
fn five_or_more_lands(q: &dyn Query, asker: &Asker) -> bool {
    lands_you_control(q, asker.controller) >= 5
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::sacrifice_to_draw("{1}, {T}, Sacrifice this land: Draw a card. Activate only if you control five or more lands.").condition(five_or_more_lands),
    ],
    ..oracle::CRYPTIC_CAVES
};
