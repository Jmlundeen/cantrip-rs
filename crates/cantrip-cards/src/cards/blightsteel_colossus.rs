use cantrip_api::*;
use cantrip_mechanics::replacements::REVEAL_AND_SHUFFLE_INTO_LIBRARY_INSTEAD_OF_GRAVEYARD;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "If Blightsteel Colossus would be put into a graveyard from anywhere, reveal Blightsteel Colossus and shuffle it
    // into its owner's library instead." (CR 614.1a, 113.6b.)
    replacements: &[REVEAL_AND_SHUFFLE_INTO_LIBRARY_INSTEAD_OF_GRAVEYARD],
    ..oracle::BLIGHTSTEEL_COLOSSUS
};
