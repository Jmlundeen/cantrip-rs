use cantrip_api::*;
use cantrip_mechanics::{lands, targets};
use cantrip_oracle as oracle;

/// "Target nontoken artifact you control with mana value X" (CR 115.1, 107.3a, 202.3). X is announced before the target is
/// chosen (`targets_after_x`), each activation's own (CR 107.3k), and judged again as the ability resolves (CR 608.2b). Before it is announced every nontoken
/// artifact you control is allowed, since some X would allow it. An artifact with {X} in its mana cost on the battlefield
/// has X of 0 (the ruling), which `Query::mana_value` gives.
fn nontoken_artifact_you_control_with_mana_value_x(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    (targets::PERMANENT.filter)(q, asker, t)
        && targets::is_nontoken_artifact(q, id)
        && q.controller(id) == asker.controller
        && targets::mana_value_is_x(q, id)
}

/// "This land becomes a copy of target nontoken artifact you control with mana value X." The artifact's copiable values as
/// the ability resolves (CR 707.2: if it is copying something, that; never counters, attachments or whether it is tapped).
/// The effect states no duration, so it lasts until the land leaves the battlefield or a later copy effect overwrites it
/// (CR 611.2a-b). It doesn't enter, so no "enters" ability applies. Nothing if the target is gone (CR 608.2b).
fn become_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(artifact) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(artifact);
    g.become_copy(item.object, item.object, values, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::ANY_COLOR_FOR_ONE,
        ability(cost("{X}, {T}"), become_copy, "{X}, {T}: This land becomes a copy of target nontoken artifact you control with mana value X.")
            .targets(&[target(nontoken_artifact_you_control_with_mana_value_x)])
            .targets_after_x(),
    ],
    ..oracle::THE_MYCOSYNTH_GARDENS
};
