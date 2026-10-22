use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{delayed, effects, targets};
use cantrip_oracle as oracle;

/// "Target instant or sorcery spell an opponent controls" (CR 115.1, 109.5): a spell on the stack, an instant or a
/// sorcery, controlled by a player other than the ability's controller.
fn instant_or_sorcery_spell_an_opponent_controls(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    let on_stack = q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id);
    on_stack
        && q.controller(id) != asker.controller
        && (q.chars(id).is(CardType::Instant) || q.chars(id).is(CardType::Sorcery))
}

const INSTANT_OR_SORCERY_SPELL_AN_OPPONENT_CONTROLS: TargetSpec = target(instant_or_sorcery_spell_an_opponent_controls);

/// "Copy target instant or sorcery spell an opponent controls twice. You may choose new targets for the copies." (CR
/// 707.10, 707.10c): two copies, neither cast, each with its own choice of new targets; nothing if the spell has left the
/// stack by then (CR 608.2b).
fn copy_twice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    g.copy_spell(spell, item.controller, true)?;
    g.copy_spell(spell, item.controller, true)?;
    Ok(())
}

/// "Create two tokens that are copies of target permanent an opponent controls. They gain haste. At the beginning of the
/// next end step, sacrifice them." (CR 707.2, 111.1, 603.7c): the two tokens are made together, copy what the permanent is
/// as the ability resolves, and are sacrificed together by the delayed ability; one that has left the battlefield is not.
fn two_hasty_copies(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let tokens = g.create_token_copies(permanent, item.controller, 2)?;
    instruction(g, |g| {
        for &token in &tokens {
            effects::gain_haste(g, item.object, token);
        }
    });
    g.end_sentence();
    delayed::at_next_end_step_about(g, item, 0, &tokens);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(copy_twice, "Copy target instant or sorcery spell an opponent controls twice. You may choose new targets for the copies.")
        .targets(&[INSTANT_OR_SORCERY_SPELL_AN_OPPONENT_CONTROLS]),
    mode(
        two_hasty_copies,
        "Create two tokens that are copies of target permanent an opponent controls. They gain haste. At the beginning of the next end step, sacrifice them.",
    )
    .targets(&[targets::PERMANENT_AN_OPPONENT_CONTROLS]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When Venser enters, choose one \u{2014}\n\u{2022} Copy target instant or sorcery spell an opponent controls twice. You may choose new targets for the copies.\n\u{2022} Create two tokens that are copies of target permanent an opponent controls. They gain haste. At the beginning of the next end step, sacrifice them.",
    )],
    delayed: &[delayed::SACRIFICE_CREATED],
    ..oracle::VENSER_FERVENT_FORGER
};
