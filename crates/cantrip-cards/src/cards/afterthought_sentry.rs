use cantrip_api::*;
use cantrip_mechanics::effects::this_gains_flying;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Whenever this creature attacks, exile up to one target card from a graveyard." (CR 115.1d, 701.13): the slot may be
/// left without a target. Nothing if that card left the graveyard before this resolves (CR 608.2b).
fn exile_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.exile_all(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{2}"), this_gains_flying, "{2}: This creature gains flying until end of turn."),
        triggered(THIS_ATTACKS, exile_card, "Whenever this creature attacks, exile up to one target card from a graveyard.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()]),
    ],
    ..oracle::AFTERTHOUGHT_SENTRY
};
