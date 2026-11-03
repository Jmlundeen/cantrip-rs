use cantrip_api::*;
use cantrip_mechanics::{delayed, prevention, targets};
use cantrip_oracle as oracle;

/// "Target blocking Wall you control gets +10/+0 until end of combat. Prevent all damage that would be dealt to it this turn. Destroy it at the beginning of the next end step." (CR 115.1).
fn target_blocking_wall_control_gets_target_0(
    q: &dyn Query,
    asker: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if cantrip_mechanics::targets::permanent(q, id) && ((q
        .state().blocks.iter().any(| b | b.blocker == id) && q.chars(id).has_subtype(q
        .words(asker.source).subtype(Subtype::Wall)) && asker.controller == q
        .controller(id)))
    )
}

/// "Target blocking Wall you control gets +10/+0 until end of combat. Prevent all damage that would be dealt to it this turn. Destroy it at the beginning of the next end step." (CR 608.2, 611.2a).
fn target_blocking_wall_control_gets(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Some(object) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        g.add_effect(item.object, object, EffectKind::Pump(10, 0), Duration::EndOfCombat);
    });
    g.end_sentence();
    prevention::this_turn(g, item, Target::Object(object), 0, Shield::All);
    g.end_sentence();
    delayed::at_next_end_step_about(g, item, 0, &[object]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[target(target_blocking_wall_control_gets_target_0)],
        target_blocking_wall_control_gets,
    ),
    delayed: &[
        delayed_trigger(delayed::destroy_carried, "Destroy it at the beginning of the next end step."),
    ],
    granted_preventions: &[prevention::ANY_DAMAGE.text("Prevent all damage that would be dealt to it this turn.")],
    ..oracle::GLYPH_OF_DESTRUCTION
};
