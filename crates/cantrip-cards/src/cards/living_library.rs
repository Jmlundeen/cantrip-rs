use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Choose target creature or planeswalker an opponent controls. Its owner shuffles it into their library." (CR 701.24a)
fn shuffle_it_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.put_into_library(&[permanent], LibraryPlace::ShuffledIn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{6}").and(SACRIFICE_THIS),
        shuffle_it_in,
        "{6}, Sacrifice this creature: Choose target creature or planeswalker an opponent controls. Its owner shuffles it into their library.",
    )
    .targets(&[targets::CREATURE_OR_PLANESWALKER_AN_OPPONENT_CONTROLS])],
    ..oracle::LIVING_LIBRARY
};
