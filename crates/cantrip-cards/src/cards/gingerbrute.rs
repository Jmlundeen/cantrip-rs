use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "{1}: This creature can't be blocked this turn except by creatures with haste."
/// The resolved activation is recorded in history for the restriction below (CR 611.2a).
fn cant_be_blocked_this_turn(_: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// The restriction applies only to this attacker after its first ability resolves this turn (CR 509.1b, 611.2a).
fn cant_be_blocked_except_by_haste(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    source != attacker
        || q.history().resolutions(source, AbilityRef { card: q.chars(source).card, index: 0 }) == 0
        || q.chars(blocker).has(Keyword::Haste)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}"), cant_be_blocked_this_turn, "{1}: This creature can't be blocked this turn except by creatures with haste."),
        ability(
            cost("{2}, {T}").and(SACRIFICE_THIS),
            |g, item| effects::gain_life(g, item, 3),
            "{2}, {T}, Sacrifice this creature: You gain 3 life.",
        ),
    ],
    block_rules: &[cant_be_blocked_except_by_haste],
    ..oracle::GINGERBRUTE
};
