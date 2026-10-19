use cantrip_api::*;
use cantrip_mechanics::{exile_target, surveil, targets};
use cantrip_oracle as oracle;

/// "Exile target creature or planeswalker that's green or blue. Surveil 1." The surveil is part of the same effect, so a
/// spell whose only target is illegal on resolution does neither (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    surveil(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::creature_or_planeswalker_of_either_color::<{ Color::Green as u8 }, { Color::Blue as u8 }>()], effect),
    ..oracle::REFUTE_DESTINY
};
