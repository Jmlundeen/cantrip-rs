use cantrip_api::*;
use cantrip_mechanics::{counter_target, effects, spent, targets};
use cantrip_oracle as oracle;

/// "Counter target spell. If the amount of mana spent to cast that spell was less than its mana value, you draw a card"
/// (CR 701.6, 601.2h, 202.3e). Asked of the spell before it is countered, though its own object answers either way (CR 608.2h);
/// a spell that is no longer a legal target means the whole spell does nothing and draws nothing (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    let drawing = spent::less_than_its_mana_value(g, spell);
    counter_target(g, item, 0);
    if drawing {
        effects::draw(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    ..oracle::UNRAVEL
};
