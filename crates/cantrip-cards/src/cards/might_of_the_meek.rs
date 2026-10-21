use cantrip_api::*;
use cantrip_mechanics::{effects, pump, targets};
use cantrip_oracle as oracle;

/// "If you control a Mouse" (CR 700.5), asked as the spell resolves (CR 608.2c): any permanent you control with the Mouse
/// creature type, the target included.
fn control_a_mouse(g: &dyn Game, item: &StackItem) -> bool {
    let mouse = g.words(item.object).subtype(Subtype::Mouse);
    g.state().battlefield().iter().any(|&id| g.controller(id) == item.controller && g.chars(id).has_subtype(mouse))
}

/// "Target creature gains trample until end of turn. It also gets +1/+0 until end of turn if you control a Mouse. Draw a
/// card." A spell whose only target has become illegal does nothing at all, so no card is drawn (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    if control_a_mouse(g, item) {
        pump(g, item.object, creature, 1, 0);
    }
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::MIGHT_OF_THE_MEEK
};
