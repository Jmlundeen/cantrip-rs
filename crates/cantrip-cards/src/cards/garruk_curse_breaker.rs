use cantrip_api::*;
use cantrip_mechanics::{effects, pump, targets, tokens};
use cantrip_oracle as oracle;

/// "Whenever a creature you control with power 4 or greater enters" (CR 603.6a): a creature under its controller's control
/// as it entered, whose power as it entered is 4 or more.
fn big_creature_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).is(CardType::Creature)
                && q.chars(*new).power.is_some_and(|p| p >= 4) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Whenever one or more creatures attack one of your opponents" (CR 508.1, 603.2c): the creatures declared as attackers
/// of one opponent are one batch, a different batch for each opponent attacked (`data[1]` is the player). Any creature
/// counts, whoever controls it; one attacking a planeswalker or battle does not. "Your" is the controller of the ability
/// that set this one up (CR 603.7d).
fn creature_attacks_one_of_your_opponents(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, defender: Target::Player(p) } if q.state().opponents(me.controller).any(|o| o == *p) => {
            Some(Fired { subject: Target::Object(*attacker), data: [0, p.0 as u32] })
        }
        _ => None,
    }
}

/// "Those creatures get +2/+2 and gain trample until end of turn" (CR 611.2): the creatures of the batch that are still on
/// the battlefield as the ability resolves.
fn attackers_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for creature in g.event_objects(item).to_vec() {
        if g.is_on_battlefield(creature) {
            pump(g, item.object, creature, 2, 2);
            g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample])), Duration::EndOfTurn);
        }
    }
    Ok(())
}

/// +2: Untap up to two target lands.
/// −3: Create a 4/4 green Beast creature token with trample.
fn create_beast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BEAST_4_4_TRAMPLE, 1)?;
    Ok(())
}

/// −4: Until your next turn (CR 611.2b), the delayed ability watches every attack on your opponents.
fn minus_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.obj(item.object).card;
    g.delayed_trigger(DelayedWhen::UntilNextTurnOf(item.controller), item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::ZONE_CHANGE, big_creature_you_control_enters), |g, item| effects::draw(g, item, 1), "Whenever a creature you control with power 4 or greater enters, draw a card."),
        ability(loyalty(2), effects::untap_targets, "+2: Untap up to two target lands.").targets(&[targets::LAND.between(0, 2)]),
        ability(loyalty(-3), create_beast, "\u{2212}3: Create a 4/4 green Beast creature token with trample."),
        ability(loyalty(-4), minus_four, "\u{2212}4: Until your next turn, whenever one or more creatures attack one of your opponents, those creatures get +2/+2 and gain trample until end of turn."),
    ],
    delayed: &[triggered(
        when_asked(EventMask::ATTACKS, creature_attacks_one_of_your_opponents),
        attackers_grow,
        "Whenever one or more creatures attack one of your opponents, those creatures get +2/+2 and gain trample until end of turn.",
    )
    .once_per_batch()
    .batch_by_data()],
    ..oracle::GARRUK_CURSE_BREAKER
};

pub static EXTRA: &[&CardDef] = &[&tokens::BEAST_4_4_TRAMPLE];
