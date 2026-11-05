use cantrip_api::*;
use cantrip_mechanics::{pump, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast a noncreature spell, Shantotto gets +X/+0 until end of turn, where X is the amount of mana spent to cast that
/// spell. If X is 4 or more, draw a card" (CR 601.2h, 611.2c): X is the mana spent on the spell the ability triggered on, which
/// resolves even if that spell was countered. The draw is the controller's, whether or not Shantotto is still on the battlefield.
fn grow_and_maybe_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    let spent = g.obj(spell).spent.total();
    pump(g, item.object, item.object, spent as i32, 0);
    if spent >= 4 {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::NONCREATURE_SPELL_YOU_CAST_ABOUT_IT,
        grow_and_maybe_draw,
        "Whenever you cast a noncreature spell, Shantotto gets +X/+0 until end of turn, where X is the amount of mana spent to cast that spell. If X is 4 or more, draw a card.",
    )],
    ..oracle::SHANTOTTO_TACTICIAN_MAGICIAN
};
