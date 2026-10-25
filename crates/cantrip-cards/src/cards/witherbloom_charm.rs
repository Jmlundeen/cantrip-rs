use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, select, targets};
use cantrip_oracle as oracle;

/// "You may sacrifice a permanent. If you do, draw two cards." The permanent is chosen as the spell resolves (CR 608.2d)
/// among those its controller controls; declining is an answer, and "if you do" is read off whether it really left.
fn sacrifice_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::may_sacrifice(g, item.controller, item.object, Selection::new(select::up_to(1)))?.is_empty() {
        g.draw(item.controller, 2);
    }
    Ok(())
}

/// "You gain 5 life."
fn gain_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 5);
    Ok(())
}

/// "Destroy target nonland permanent with mana value 2 or less." (CR 701.8)
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(sacrifice_to_draw, "You may sacrifice a permanent. If you do, draw two cards."),
    mode(gain_life, "You gain 5 life."),
    mode(destroy, "Destroy target nonland permanent with mana value 2 or less.")
        .targets(&[target(targets::nonland_permanent_with_mana_value_at_most::<2>)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::WITHERBLOOM_CHARM };
