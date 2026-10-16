use cantrip_api::*;
use cantrip_mechanics::melee;
use cantrip_oracle as oracle;

/// "If a creature you control attacking causes a triggered ability of a permanent you control to trigger, that ability
/// triggers an additional time." (CR 603.2d, 508.2a): an ability whose trigger is a creature being declared as an attacker
/// ("whenever [this] attacks", "whenever you attack"), the attacking creature being one you control - an opponent's attack
/// is not it, though it triggers an ability of yours. Wulfgar's own melee among them (the rulings).
fn your_creature_attacking(q: &dyn Query, asker: &Asker, event: &Event, triggering: &Triggering) -> bool {
    let you = asker.controller;
    triggering.of_a_permanent_controlled_by(you)
        && match *event {
            Event::Attacks { attacker, .. } => q.controller(attacker) == you,
            Event::AttackersDeclared { player, .. } => player == you,
            _ => false,
        }
}

pub static CARD: CardDef = CardDef {
    abilities: &[melee::TRIGGER],
    additional_triggers: &[your_creature_attacking],
    ..oracle::WULFGAR_OF_ICEWIND_DALE
};
