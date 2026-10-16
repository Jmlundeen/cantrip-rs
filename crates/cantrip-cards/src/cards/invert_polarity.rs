use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Choose target spell, then flip a coin." The flip is a two-sided die (CR 705.1), won on its first face, as Mana Crypt's
/// is read. Win: gain control of the spell and choose new targets for it (CR 112.2, 115.7); lose: counter it. A spell that
/// has left the stack is not flipped for.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    if !g.flip_coin(item.controller) {
        g.counter(spell);
        return Ok(());
    }
    g.change_controller(spell, item.controller);
    g.choose_new_targets(spell, item.controller)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    ..oracle::INVERT_POLARITY
};
