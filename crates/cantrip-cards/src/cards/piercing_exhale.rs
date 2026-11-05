use cantrip_api::*;
use cantrip_mechanics::{behold, effects, fight, targets};
use cantrip_oracle as oracle;

/// "Target creature you control deals damage equal to its power to target creature or planeswalker. If a Dragon was beheld,
/// surveil 2." (CR 120.3, 609.7, 701.4a, 701.25.) The surveil is not part of the damage, so it happens even when the damage is
/// not dealt because one of the targets is illegal (CR 608.2b) as long as the other still is.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    fight::targets_bite(g, item, 0, 1);
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    if behold::beheld(item) {
        effects::surveil(g, item, 2)?;
    }
    Ok(())
}

/// "As an additional cost to cast this spell, you may behold a Dragon" (CR 701.4a, 601.2b): an optional cost announced before the
/// targets, which are chosen after it. Caustic Exhale, Dispelling Exhale and Osseous Exhale also behold a Dragon.
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_OR_PLANESWALKER], effect),
    optional_cost: Some(behold::optional(behold::dragon, "Dragon")),
    labels: &["Cast without beholding", "Behold a Dragon: choose a Dragon you control or reveal one from your hand"],
    ..oracle::PIERCING_EXHALE
};
