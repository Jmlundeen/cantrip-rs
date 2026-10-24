use cantrip_api::*;
use cantrip_api::Subtype;
use cantrip_oracle as oracle;

/// "You may have this creature enter as a copy of a creature you control, except it's a Shapeshifter Rogue in addition
/// to its other types." The copied values, including existing copy exceptions, gain both subtypes (CR 614.1c, 707.2).
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy(g, entry, |c| c.is(CardType::Creature), |_, mut copy| {
        copy.extra_subtypes = copy.extra_subtypes.with(Subtype::Shapeshifter).with(Subtype::Rogue);
        copy
    })
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(enter_as_copy), ..oracle::GLASSPOOL_MIMIC };
