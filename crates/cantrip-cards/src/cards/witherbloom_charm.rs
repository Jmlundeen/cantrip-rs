use cantrip_api::*;
use cantrip_mechanics::{choose, destroy_target};
use cantrip_oracle as oracle;

/// "You may sacrifice a permanent. If you do, draw two cards." The permanent is chosen as the spell resolves (CR 608.2d)
/// among those its controller controls; declining is an answer, and "if you do" is read off whether it really left.
fn sacrifice_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let permanents = g.permanents_of(item.controller);
    let Some(&permanent) = choose::cards(g, item.controller, item.object, Purpose::Sacrifice, permanents, 1, true)?.first() else {
        return Ok(());
    };
    if !g.sacrifice(&[permanent]).is_empty() {
        g.draw(item.controller, 2);
    }
    Ok(())
}

/// "You gain 5 life."
fn gain_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 5);
    Ok(())
}

/// "Target nonland permanent with mana value 2 or less" (CR 115.1, 202.3: X on the battlefield is 0), whoever controls it.
fn nonland_permanent_with_mana_value_two_or_less(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id)
        && !q.chars(id).is(CardType::Land) && q.mana_value(id) <= 2)
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
        .targets(&[target(nonland_permanent_with_mana_value_two_or_less)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::WITHERBLOOM_CHARM };
