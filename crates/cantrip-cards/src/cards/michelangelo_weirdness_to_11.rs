use cantrip_api::*;
use cantrip_mechanics::replacements::{plus_one_counters_applies, plus_one_counters_apply};
use cantrip_mechanics::tokens::MUTAGEN;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on
/// it instead." (CR 122.6, 614.1a.)
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    plus_one_counters_applies(q, asker, proposal, &[CardType::Creature])
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    plus_one_counters_apply(proposal);
}

fn create_mutagen(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &MUTAGEN, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, create_mutagen, "When Michelangelo enters, create a Mutagen token.")],
    replacements: &[replacement(ReplacementKind::Counters, applies, apply, "puts one more +1/+1 counter")],
    ..oracle::MICHELANGELO_WEIRDNESS_TO_11
};

/// The Mutagen token it makes.
pub static EXTRA: &[&CardDef] = &[&MUTAGEN];
