use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may have this creature enter as a copy of any creature on the battlefield, except it has changeling." A choice, not a
/// target (CR 608.2d), so hexproof and protection narrow nothing and an opponent's creature is as available as your own. With
/// no other creature out nothing is asked, and with one it is taken without asking which. The exception is part of the copy's
/// own copiable values (CR 707.9b): changeling (CR 702.73a) is a characteristic-defining ability, so the copy is every creature
/// type, and a later copy of it has changeling too.
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy(g, entry, |c| c.is(CardType::Creature), |_, copy| copy.with_keyword(Keyword::Changeling))
}

pub static CARD: CardDef = CardDef { enters: Some(enter_as_copy), ..oracle::OMNI_CHANGELING };
