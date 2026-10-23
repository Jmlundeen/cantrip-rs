use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target spell you don't control" (CR 115.1, 109.5): a spell on the stack whose controller is another player.
fn spell_you_dont_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if q.controller(id) != asker.controller
            && q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id))
}

/// "Return target spell you don't control to its owner's hand." Not a counter, so a spell that can't be countered
/// goes too. A spell whose target is gone is not returned (CR 608.2b).
fn return_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(spell) = targets::object(g, item, 0) {
        g.remove_spell(spell, CounterTo::Hand);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(return_spell, "Return target spell you don't control to its owner's hand.").targets(&[target(spell_you_dont_control)]),
    mode(effects::return_first_target_to_hand, "Return target nonland permanent to its owner's hand.")
        .targets(&[targets::NONLAND_PERMANENT]),
])
.choose(0, 1);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        cantrip_mechanics::triggers::YOU_CAST_A_SPELL,
        &MODAL,
        "Whenever you cast a spell, choose up to one \u{2014} \u{2022} Return target spell you don't control to its owner's hand. \u{2022} Return target nonland permanent to its owner's hand.",
    )],
    ..oracle::HULLBREAKER_HORROR
};
