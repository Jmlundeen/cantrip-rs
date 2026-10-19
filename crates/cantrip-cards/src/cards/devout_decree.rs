use cantrip_api::*;
use cantrip_mechanics::{exile_target, scry, targets};
use cantrip_oracle as oracle;

/// "Exile target creature or planeswalker that's black or red. Scry 1." The scry is part of the same effect, so a spell
/// whose only target is illegal on resolution does neither (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    scry(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::creature_or_planeswalker_of_either_color::<{ Color::Black as u8 }, { Color::Red as u8 }>()], effect),
    ..oracle::DEVOUT_DECREE
};
