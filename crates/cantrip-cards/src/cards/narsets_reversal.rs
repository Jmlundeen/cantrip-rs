use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Copy target instant or sorcery spell, then return it to its owner's hand. You may choose new targets for the copy."
/// The copy is made first (CR 707.10), and only then is the original taken off the stack: not a counter, so a spell
/// that can't be countered goes too (CR 701.6); a copy returned to hand ceases to exist (CR 707.10a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    g.copy_spell(spell, item.controller, true)?;
    g.end_sentence();
    g.remove_spell(spell, CounterTo::Hand);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::INSTANT_OR_SORCERY_SPELL], effect), ..oracle::NARSETS_REVERSAL };
