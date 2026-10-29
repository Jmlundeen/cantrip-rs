use cantrip_api::*;
use cantrip_mechanics::tokens::{self, MUTAGEN};
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

/// "Whenever Genghis Frog or another Mutant you control enters" (CR 603.6a, 201.5): this Frog, whatever it is as it enters (Supper
/// for Spiders can bring it back as a Food artifact), or a Mutant under your control as it entered, a token too.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::this_or_another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Mutant) }>),
        tokens::create_mutagen,
        "Whenever Genghis Frog or another Mutant you control enters, create a Mutagen token.",
    )],
    ..oracle::GENGHIS_FROG
};

/// The Mutagen token it makes.
pub static EXTRA: &[&CardDef] = &[&MUTAGEN];
