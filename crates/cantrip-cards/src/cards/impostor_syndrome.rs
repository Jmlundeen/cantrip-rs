use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever a nontoken creature you control deals combat damage to a player" (CR 510.2, 603.2): one trigger for each creature
/// and each damage dealt, with the creature the trigger's subject ("it"). Whether it is a token is judged as the damage is
/// dealt (CR 111.7); damage of 0 is no damage dealt (CR 120.8), and damage to a creature or planeswalker is no damage to a player.
fn nontoken_creature_connects(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source, target: Target::Player(_), kind, .. }
            if kind.is_combat() && q.controller(*source) == me.controller && q.chars(*source).is(CardType::Creature) && !q.obj(*source).token =>
        {
            Fired::about(Target::Object(*source))
        }
        _ => None,
    }
}

/// "Create a token that's a copy of it, except it isn't legendary." (CR 707.2, 707.9b, 111.1): the copiable values of the
/// creature that dealt the damage, as it last existed if it has left the battlefield by now (CR 608.2h), for the ability's
/// controller. An {X} in its mana cost is 0 (CR 107.3f).
fn copy_it_nonlegendary(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    let values = g.copiable_values(creature).nonlegendary();
    g.create_token_copies_of(values, item.controller, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, nontoken_creature_connects),
        copy_it_nonlegendary,
        "Whenever a nontoken creature you control deals combat damage to a player, create a token that's a copy of it, except it isn't legendary.",
    )],
    ..oracle::IMPOSTOR_SYNDROME
};
