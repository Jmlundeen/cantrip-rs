use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Count Mountains on resolution and deal both amounts simultaneously (CR 608.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mountain = g.words(item.object).subtype(Subtype::Mountain);
    let count = g.state().battlefield().iter()
        .filter(|&&id| g.controller(id) == item.controller && g.chars(id).has_subtype(mountain))
        .count();
    let target = g.state().target(item, 0);
    g.damage_together(&[
        (item.object, target, count as u32),
        (item.object, Target::Player(item.controller), count.div_ceil(2) as u32),
    ]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT_OR_PLANESWALKER], effect),
    ..oracle::ETERNAL_FLAME
};
