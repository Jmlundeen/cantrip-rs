use cantrip_api::*;
use cantrip_mechanics::replacements::REVEAL_AND_SHUFFLE_INTO_LIBRARY_INSTEAD_OF_GRAVEYARD;
use cantrip_mechanics::statics::itself;
use cantrip_oracle as oracle;

/// "Protection from everything" (CR 702.16j, layer 6): it can't be blocked, enchanted, equipped or targeted, and damage to
/// it is prevented; the core does each of those.
fn protection_from_everything(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.protection_everything = true;
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, itself, protection_from_everything, "Protection from everything")],
    // "If Progenitus would be put into a graveyard from anywhere, reveal Progenitus and shuffle it into its owner's
    // library instead." (CR 614.1a, 113.6b.)
    replacements: &[REVEAL_AND_SHUFFLE_INTO_LIBRARY_INSTEAD_OF_GRAVEYARD],
    ..oracle::PROGENITUS
};
