use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever a nontoken artifact creature you control deals combat damage to a player" (CR 510.2, 603.2): one trigger for each such
/// creature and player it hurt, "that player" fixed as the trigger's subject. The creature is an artifact creature and no token as
/// it dealt the damage (CR 603.10), wherever it is now.
fn nontoken_artifact_creature_you_control_deals_combat_damage_to_player(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Player(p), kind, .. }
            if kind.is_combat()
                && q.controller(*from) == me.controller
                && !q.obj(*from).token
                && q.chars(*from).is(CardType::Creature)
                && q.chars(*from).is(CardType::Artifact) =>
        {
            Fired::about(Target::Player(*p))
        }
        _ => None,
    }
}

/// "That player gets two poison counters" (CR 122.1f, 704.5c).
fn that_player_gets_two_poison_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(player) = item.subject {
        g.give_poison(player, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, nontoken_artifact_creature_you_control_deals_combat_damage_to_player),
        that_player_gets_two_poison_counters,
        "Whenever a nontoken artifact creature you control deals combat damage to a player, that player gets two poison counters.",
    )],
    ..oracle::VIRULENT_SILENCER
};
