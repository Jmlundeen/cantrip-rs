use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "a creature source": the types of the source a copied ability may come from.
const CREATURE_SOURCE: u16 = TypeSet::of(&[CardType::Creature]).0;

/// "Copy target activated or triggered ability you control from a creature source. You may choose new targets for the copy."
/// (CR 707.10, 707.10c) The copy is neither activated nor triggered; its only target no longer legal, the
/// ability does not resolve at all (CR 608.2b).
fn copy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.copy_ability(item.target(0), item.controller, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}, {T}"),
        copy_it,
        "{1}, {T}: Copy target activated or triggered ability you control from a creature source. You may choose new targets for the copy.",
    )
    .targets(&[target(targets::your_ability_from_a_source_of_types::<CREATURE_SOURCE>)])],
    ..oracle::ECHO_PERCEPTIVE_PRODIGY
};
