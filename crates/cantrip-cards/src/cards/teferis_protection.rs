use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Until your next turn, your life total can't change and you gain protection from everything." (CR 119.7, 119.8,
/// 702.16j, 611.2b): two effects about the spell's controller, which last until their next turn begins. "All permanents you
/// control phase out." (CR 702.26b): together, and with them, indirectly, whatever is attached to them (CR 702.26g); they
/// phase in during their controller's next untap step (CR 502.1). "Exile Teferi's Protection." (CR 608.2c): its last
/// instruction, so it goes to exile rather than to its owner's graveyard.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    instruction(g, |g| {
        g.add_player_effect(you, item.object, EffectKind::LifeTotalCantChange(you), Duration::UntilNextTurnOf(you));
        g.add_player_effect(you, item.object, EffectKind::PlayerProtectionFromEverything, Duration::UntilNextTurnOf(you));
    });
    g.end_sentence();
    let permanents = g.permanents_of(you);
    g.phase_out(&permanents);
    g.end_sentence();
    g.exile(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::TEFERIS_PROTECTION };
