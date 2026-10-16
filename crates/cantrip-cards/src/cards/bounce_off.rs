use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature or Vehicle" (CR 115.1, 205.3g): a permanent on the battlefield that is a creature, or a Vehicle
/// whether or not it is crewed into a creature right now.
fn creature_or_vehicle(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) || matches!(t, Target::Object(id) if targets::permanent(q, id) && q.chars(id).has_subtype(Subtype::Vehicle))
}

/// "Return target creature or Vehicle to its owner's hand."
pub static CARD: CardDef = CardDef { spell: spell(&[target(creature_or_vehicle)], effects::return_first_target_to_hand), ..oracle::BOUNCE_OFF };
