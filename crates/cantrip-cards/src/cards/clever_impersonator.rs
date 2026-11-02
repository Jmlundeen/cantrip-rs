use cantrip_api::*;
use cantrip_mechanics::copy;
use cantrip_oracle as oracle;

/// "You may have this permanent enter as a copy of any nonland permanent on the battlefield." A replacement effect on its entering
/// (CR 614.1c, 707.2): a choice, not a target (CR 608.2d), with no exception to the copy.
fn enter_as_copy(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    copy::enter_as_copy(g, entering, |c| !c.is(CardType::Land), |_, c| c)
}

pub static CARD: CardDef = CardDef {
    enters_as_copy: Some(enter_as_copy),
    ..oracle::CLEVER_IMPERSONATOR
};
