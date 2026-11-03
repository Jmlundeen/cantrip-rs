use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever this creature blocks a creature, that creature can't attack during its controller's next turn." (CR 608.2, 611.2a).
fn creature_cant_attack_during_controllers(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Target::Object(attacker) = item.subject else { return Ok(()) };
    instruction(g, |g| {
        g.add_effect(item.object, attacker, EffectKind::CantAttack, Duration::DURING_ITS_CONTROLLERS_NEXT_TURN);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::BLOCKS, triggers::this_blocks_a_creature),
            creature_cant_attack_during_controllers,
            "Whenever this creature blocks a creature, that creature can't attack during its controller's next turn.",
        ),
    ],
    ..oracle::WALL_OF_DUST
};
