use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

/// "{T}: Gain control of target creature whose controller controls an Island for as long as you control this creature and this creature remains tapped." (CR 115.1).
fn gain_control_target_creature_whose_target_0(
    q: &dyn Query,
    asker: &Asker,
    t: Target,
) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && q.chars(id).is(CardType::Creature)
        && lands::controls_land_type(q, q.controller(id), q.words(asker.source).subtype(Subtype::Island)))
}

/// "{T}: Gain control of target creature whose controller controls an Island for as long as you control this creature and this creature remains tapped." (CR 608.2, 611.2a).
fn gain_control_target_creature_whose(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(
        g,
        |g| {
            g.add_effect(
                item.object,
                target,
                EffectKind::Control(item.controller),
                Duration::WhileSourceTappedAndControlled,
            );
        },
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        state_triggered(
            lands::you_control_no_islands,
            effects::sacrifice_self,
            "When you control no Islands, sacrifice this creature.",
        ),
        ability(
                TAP,
                gain_control_target_creature_whose,
                "{T}: Gain control of target creature whose controller controls an Island for as long as you control this creature and this creature remains tapped.",
            )
            .targets(&[target(gain_control_target_creature_whose_target_0)]),
    ],
    may_not_untap: true,
    ..oracle::SEASINGER
};
