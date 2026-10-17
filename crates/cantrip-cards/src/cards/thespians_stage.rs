use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{2}, {T}: This land becomes a copy of target land, except it has this ability." The target's copiable values as the
/// ability resolves (CR 707.2: its printed card, or what it is copying with that copy's exceptions; never counters, an
/// animation or whether it is tapped), with this very ability added as an exception, so it is a copiable value too
/// (CR 707.9a-b) and a later copy of this land has it. The effect states no duration, so it lasts until the land leaves the
/// battlefield or a later copy effect overwrites it (CR 611.2a-b; the ruling), and the land keeps its counters and stays
/// tapped. It doesn't enter, so no "enters" ability or "enters with" replacement of the copied land applies (the ruling).
/// Nothing happens if the target is gone (CR 608.2b: the core doesn't resolve it).
fn become_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(land) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(land).with_ability(item.ability);
    g.become_copy(item.object, item.object, values, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}, {T}"), become_copy, "{2}, {T}: This land becomes a copy of target land, except it has this ability.").targets(&[targets::LAND]),
    ],
    ..oracle::THESPIANS_STAGE
};
