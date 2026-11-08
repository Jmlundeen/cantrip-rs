use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{4}, {T}: Target creature can't be blocked this turn." (CR 509.1b): the target is any creature - its controller's or an
/// opponent's - and the restriction lasts until the turn ends, so it can attack unblocked in the same turn and is back to
/// normal on the next one. "{T}: Add {C}" is the land's colorless mana ability (CR 605.1a).
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{4}").and(TAP),
            effects::target_creature_cant_be_blocked_this_turn,
            "{4}, {T}: Target creature can't be blocked this turn.",
        )
        .targets(&[targets::CREATURE]),
    ],
    ..oracle::ROGUES_PASSAGE
};
