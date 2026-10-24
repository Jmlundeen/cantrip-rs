use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may have this creature enter as a copy of any creature on the battlefield." A replacement effect on its entering
/// (CR 614.1c, 707.2): a choice, not a target (CR 608.2d), so hexproof and protection narrow nothing and an opponent's
/// creature is as available as your own. What is a creature is judged as it is now, so an animated land may be chosen,
/// but only its copiable values are taken (CR 707.2): the printed card, or what it is itself copying with that copy's
/// exceptions (CR 707.9b), never counters, Auras, tapped state or other effects. With nothing to copy nothing is asked,
/// and declining leaves it the printed 0/0 that state-based actions put into the graveyard (CR 704.5f). It copies
/// with no exception.
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy(g, entry, |c| c.is(CardType::Creature), |_, copy| copy)
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(enter_as_copy), ..oracle::CLONE };
