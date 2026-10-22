use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Choose target opponent. Until that player's next turn, they gain protection from everything and their life total
/// can't change. All nonland permanents they control phase out." (CR 115.1: the target is that opponent, never the
/// caster; CR 119.7, 119.8, 702.16j, 611.2b: the two effects about them last until their next turn begins; CR 702.26b:
/// their nonland permanents phase out together, and phase in during their next untap step, CR 502.1). "Exile Teferi's
/// Reproach." (CR 608.2c): its last instruction, so it goes to exile rather than to its owner's graveyard.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    instruction(g, |g| {
        g.add_player_effect(opponent, item.object, EffectKind::LifeTotalCantChange(opponent), Duration::UntilNextTurnOf(opponent));
        g.add_player_effect(opponent, item.object, EffectKind::PlayerProtectionFromEverything, Duration::UntilNextTurnOf(opponent));
    });
    g.end_sentence();
    let permanents: Vec<ObjectId> =
        g.permanents_of(opponent).into_iter().filter(|&id| !g.chars(id).is(CardType::Land)).collect();
    g.phase_out(&permanents);
    g.end_sentence();
    g.exile(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::TEFERIS_REPROACH
};
