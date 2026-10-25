use cantrip_api::*;
use cantrip_mechanics::replacements::PLUS_ONE_COUNTERS_ON_YOUR_CREATURES;
use cantrip_mechanics::tokens::MUTAGEN;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn create_mutagen(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &MUTAGEN, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, create_mutagen, "When Michelangelo enters, create a Mutagen token.")],
    replacements: &[PLUS_ONE_COUNTERS_ON_YOUR_CREATURES],
    ..oracle::MICHELANGELO_WEIRDNESS_TO_11
};

/// The Mutagen token it makes.
pub static EXTRA: &[&CardDef] = &[&MUTAGEN];
