use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Return up to one target creature card from your graveyard to the battlefield with a lifelink counter on it": the
/// counter is one it enters with (CR 614.1c, 122.6), and grants lifelink through the layers (CR 122.1b).
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield_with_counters(card, item.controller, false, &[(CounterKind::Lifelink, 1)])?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    miracle: Some(mana(1, 0, 0, 1, 0, 0)),
    abilities: &[triggered(THIS_ENTERS, reanimate, "When this creature enters, return up to one target creature card from your graveyard to the battlefield with a lifelink counter on it.")
        .targets(&[TargetSpec { min: 0, ..cantrip_mechanics::targets::CREATURE_CARD_IN_YOUR_GRAVEYARD }])],
    ..oracle::METAMORPHOSIS_FANATIC
};
