use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

/// "Target attacking creature you control" (CR 115.1).
fn attacking_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(
        t, Target::Object(id) if targets::permanent(q, id) && q.is_attacking(id)
        && q.chars(id).is(CardType::Creature) && asker.controller == q.controller(id)
    )
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{2}").and(TAP),
            prevention::untap_target_and_prevent_combat_damage_to_and_by_it,
            "{2}, {T}: Untap target attacking creature you control. Prevent all combat damage that would be dealt to and dealt by that creature this turn.",
        )
        .targets(&[target(attacking_creature_you_control)]),
    ],
    granted_preventions: &[prevention::COMBAT_DAMAGE_DEALT_TO_AND_BY],
    ..oracle::EBONY_HORSE
};
