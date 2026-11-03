use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// Gain control of the target artifact for as long as you control this creature (CR 611.2b).
fn gain_control(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(object) = targets::object(g, item, 0) else { return Ok(()) };
    let duration = Duration::AsLongAs(g.card_id(&CARD), 0);
    instruction(g, |g| {
        g.add_effect(item.object, object, EffectKind::Control(item.controller), duration);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}{R}{R}").and(TAP),
        gain_control,
        "{1}{R}{R}, {T}: Gain control of target artifact for as long as you control this creature.",
    )
    .targets(&[targets::ARTIFACT])],
    lasting: &[effects::YOU_CONTROL_SOURCE],
    ..oracle::ALADDIN
};
