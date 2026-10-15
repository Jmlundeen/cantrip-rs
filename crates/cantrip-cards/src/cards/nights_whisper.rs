use cantrip_api::*;
use cantrip_oracle as oracle;

/// You draw two cards and lose 2 life.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    g.lose_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::NIGHTS_WHISPER };
