use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Exile target nonland permanent. Its controller loses 3 life and you gain 3 life." If the target is illegal, nothing
/// happens (CR 608.2b). Its controller is the one it had as it was exiled (CR 113.7a, 608.2h); they lose the life even if
/// a replacement effect kept the permanent from being exiled. (It can't be countered: the card's keyword.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(permanent);
    g.exile(permanent);
    g.lose_life(controller, 3);
    g.gain_life(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::NONLAND_PERMANENT], effect }),
    ..oracle::INEVITABLE_DEFEAT
};
