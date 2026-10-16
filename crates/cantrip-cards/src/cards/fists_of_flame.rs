use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// Draw a card. Until end of turn, target creature gains trample and gets +1/+0 for each card you've drawn this turn:
/// counted once, as the spell resolves and after its own draw (CR 608.2c), so a later draw adds nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let drawn = g.history().drawn(item.controller) as i16;
    pump(g, item.object, creature, drawn, 0);
    g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample])), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::FISTS_OF_FLAME
};
