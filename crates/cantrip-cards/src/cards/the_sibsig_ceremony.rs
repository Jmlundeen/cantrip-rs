use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, tokens};
use cantrip_oracle as oracle;

/// "Creature spells you cast cost {2} less to cast." (CR 601.2f, 604.1).
fn creature_spells_cast_cost_2_rule(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    cost_reduction::spells_you_cast(q, source, player, costed, CardType::Creature, 2)
}

/// "Whenever a creature you control enters, if you cast it" (CR 603.2, 603.4, 603.6a): a creature under its controller's control
/// that entered as the spell that resolved, which the history says its controller cast. The creature is the trigger's subject.
fn creature_you_control_enters_if_you_cast_it(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { old, new, to, reason: ZoneReason::Resolve, .. } = event else { return None };
    if to.is(ZoneKind::Battlefield)
        && q.controller(*new) == me.controller
        && q.chars(*new).is(CardType::Creature)
        && q.history().cast_of(*old).is_some_and(|cast| cast.player == me.controller)
    {
        return Fired::about(Target::Object(*new));
    }
    None
}

/// "Destroy that creature, then create a 2/2 black Zombie Druid creature token" (CR 701.8, 111.1, 608.2c): the token is made
/// whether or not the creature was destroyed (indestructible) or is still there.
fn destroy_it_then_create_a_zombie_druid(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(entered) = item.subject.object() else { return Ok(()) };
    g.destroy(entered);
    tokens::create(g, item.controller, &tokens::ZOMBIE_DRUID, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[creature_spells_cast_cost_2_rule],
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, creature_you_control_enters_if_you_cast_it),
        destroy_it_then_create_a_zombie_druid,
        "Whenever a creature you control enters, if you cast it, destroy that creature, then create a 2/2 black Zombie Druid creature token.",
    )],
    ..oracle::THE_SIBSIG_CEREMONY
};
