use cantrip_api::*;
use cantrip_mechanics::lands_you_control;
use cantrip_oracle as oracle;

/// "Activate only if you control five or more lands" (CR 602.5b): checked as the ability is activated, before its cost is
/// paid, so the temple itself counts as one of the five and tapping it does not undo the activation.
fn five_or_more_lands(q: &dyn Query, asker: &Asker) -> bool {
    lands_you_control(q, asker.controller) >= 5
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}. Activate only if you control five or more lands.")
        .condition(five_or_more_lands)],
    ..oracle::TEMPLE_OF_THE_FALSE_GOD
};
