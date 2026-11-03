use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{T}: Return another target permanent you control to its owner's hand. Activate only during your turn." A target
/// that has become illegal is left alone (CR 608.2b).
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.return_to_hand(&[permanent]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        return_it,
        "{T}: Return another target permanent you control to its owner's hand. Activate only during your turn.",
    )
    .targets(&[targets::ANOTHER_PERMANENT_YOU_CONTROL])
    .timing(Timing::OwnTurn)],
    ..oracle::RESCUE_GIRL_FIRST_RESPONDER
};
