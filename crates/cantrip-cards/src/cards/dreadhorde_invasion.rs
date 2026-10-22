use cantrip_api::*;
use cantrip_mechanics::amass::amass;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{self, YOUR_UPKEEP};
use cantrip_oracle as oracle;

/// The 0/0 black Zombie Army creature token amass Zombies makes when its controller has no Army (CR 701.47a).
pub static ZOMBIE_ARMY: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::Black)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(0),
    toughness: Some(0),
    ..token!(Zombie Army)
};

/// "At the beginning of your upkeep, you lose 1 life and amass Zombies 1."
fn lose_and_amass(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_life(item.controller, 1);
    amass(g, item.controller, item.object, &ZOMBIE_ARMY, Subtype::Zombie, 1)
}

/// "Whenever a Zombie token you control with power 6 or greater attacks" (CR 603.2, 508.3a): a creature its controller has
/// attacks (the shared matcher, about that attacker), and as the trigger event is checked it is a token, a Zombie, with power
/// 6 or more - the power is read then, so a change later in the turn does not undo or add the lifelink (rulings 2019-05-03).
fn zombie_token_attacks(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let fired = triggers::creature_you_control_attacks(q, me, event)?;
    let Target::Object(attacker) = fired.subject else { return None };
    let c = q.chars(attacker);
    let zombie = q.obj(attacker).token
        && c.has_subtype(q.words(me.source).subtype(Subtype::Zombie))
        && c.power.is_some_and(|power| power >= 6);
    zombie.then_some(fired)
}

/// "it gains lifelink until end of turn."
fn gain_lifelink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(attacker) = item.subject {
        effects::gain_lifelink_until_end_of_turn(g, item.object, attacker);
    }
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&ZOMBIE_ARMY];

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOUR_UPKEEP, lose_and_amass, "At the beginning of your upkeep, you lose 1 life and amass Zombies 1. (Put a +1/+1 counter on an Army you control. It's also a Zombie. If you don't control an Army, create a 0/0 black Zombie Army creature token first.)"),
        triggered(when_asked(EventMask::ATTACKS, zombie_token_attacks), gain_lifelink, "Whenever a Zombie token you control with power 6 or greater attacks, it gains lifelink until end of turn."),
    ],
    ..oracle::DREADHORDE_INVASION
};
