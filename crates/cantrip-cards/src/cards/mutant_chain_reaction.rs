use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::tokens::{self, MUTAGEN};
use cantrip_oracle as oracle;

/// "Destroy up to one target ... Create a Mutagen token." (CR 701.8a): two sentences; the token comes whether or not anything
/// was targeted, or destroyed.
fn destroy_then_mutagen(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(target) = targets::object(g, item, 0) {
        g.destroy(target);
    }
    g.end_sentence();
    tokens::create_mutagen(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_ENCHANTMENT_OR_CREATURE_WITH_FLYING.optional()], destroy_then_mutagen),
    ..oracle::MUTANT_CHAIN_REACTION
};

/// The Mutagen token it makes.
pub static EXTRA: &[&CardDef] = &[&MUTAGEN];
