use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Return target creature to its owner's hand. If its mana value was 3 or less, scry 1." (CR 400.3, 608.2b, 701.22)
/// "Was": its mana value as it last existed on the battlefield (CR 608.2h), read before it leaves. A target no longer
/// legal makes the spell fizzle (CR 608.2b), so neither instruction happens.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let small = g.mana_value(creature) <= 3;
    effects::return_first_target_to_hand(g, item)?;
    g.end_sentence();
    if small {
        effects::scry(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::FADING_HOPE
};
