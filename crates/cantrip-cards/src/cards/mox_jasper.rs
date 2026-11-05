use cantrip_api::*;
use cantrip_mechanics::permanents_with_subtype_you_control;
use cantrip_oracle as oracle;

/// "Activate only if you control a Dragon" (CR 602.5, 602.5b), on a mana ability (CR 605.1a): counted through the layers each time the ability is offered, so a
/// changeling or any permanent with the subtype counts, and only the Mox's controller's do. A restriction on activating, not on
/// having: the ability is on the Mox whatever it controls.
fn control_a_dragon(q: &dyn Query, asker: &Asker) -> bool {
    permanents_with_subtype_you_control(q, asker.controller, q.words(asker.source).subtype(Subtype::Dragon)) > 0
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color. Activate only if you control a Dragon.").condition(control_a_dragon)],
    ..oracle::MOX_JASPER
};
