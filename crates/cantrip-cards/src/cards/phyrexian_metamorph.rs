use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may have this creature enter as a copy of any artifact or creature on the battlefield, except it's an
/// artifact in addition to its other types." A choice, not a target (CR 608.2d), so hexproof and protection narrow
/// nothing and an opponent's permanent is as available as your own. With nothing to copy nothing is asked, and with
/// one candidate it is taken without asking. The exception is part of the copy's own copiable values (CR 707.9b), so
/// a later copy of the Metamorph is an artifact too.
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy(
        g,
        entry,
        |c| c.is(CardType::Artifact) || c.is(CardType::Creature),
        |_, copy| copy.with_type(CardType::Artifact),
    )
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(enter_as_copy), ..oracle::PHYREXIAN_METAMORPH };
