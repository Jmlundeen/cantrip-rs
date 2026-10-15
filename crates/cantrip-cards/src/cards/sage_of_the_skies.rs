use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If you've cast another spell this turn" (CR 603.4), asked of the event history: this spell's own cast is one of the
/// records, so another is a second.
fn cast_another_spell_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.history().casts_by(asker.controller).count() > 1
}

/// Copy this spell (CR 707.10): a permanent spell's copy becomes a token as it resolves (CR 707.10f).
fn copy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.copy_spell(item.object, item.controller, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When you cast this spell, if you've cast another spell this turn, copy this spell. (The copy becomes a token.)",
        listens: EventMask::SPELL_CAST,
        matches: cantrip_mechanics::triggers::cast_this_spell,
        condition: Some(cast_another_spell_this_turn),
        effect: copy_it,
        anywhere: true,
        ..TriggeredDef::BLANK
    })],
    ..oracle::SAGE_OF_THE_SKIES
};
