use cantrip_api::*;
use cantrip_mechanics::{mana, targets};
use cantrip_oracle as oracle;

/// "{4}{U}: This land becomes a copy of target creature you control until end of turn, except it isn't legendary and it's a
/// Robot artifact in addition to its other types." The creature's copiable values as the ability resolves (CR 707.2, 707.3),
/// with those exceptions, so they are copiable values too (CR 707.9a-b); "Robot" is a word of the card's text (CR 612). The
/// land doesn't enter, so no "enters" ability applies, and it keeps its counters and tapped state. An opponent's creature is
/// no target (CR 115.1), and with the target gone the ability does nothing (CR 608.2b). The effect ends in the cleanup step
/// (CR 514.2), when the land is a land again.
fn become_a_robot_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g
        .copiable_values(creature)
        .nonlegendary()
        .with_type(CardType::Artifact)
        .with_subtype(g.words(item.object).subtype(Subtype::Robot));
    g.become_copy(item.object, item.object, values, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::U), "{T}: Add {U}."),
        ability(
            cost("{4}{U}"),
            become_a_robot_copy,
            "{4}{U}: This land becomes a copy of target creature you control until end of turn, except it isn't legendary and it's a Robot artifact in addition to its other types.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    enters: Some(mana::enters_tapped),
    ..oracle::SHORE_LEAVE_PLANET
};
