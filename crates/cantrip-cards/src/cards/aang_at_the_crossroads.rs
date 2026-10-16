use cantrip_api::*;
use cantrip_mechanics::triggers::{BEGINNING_OF_YOUR_COMBAT, THIS_ENTERS};
use cantrip_mechanics::{choose, earthbend, effects, library, statics, top_of_library};
use cantrip_oracle as oracle;

/// "Look at the top five cards of your library. You may put a creature card with mana value 4 or less from among them onto
/// the battlefield. Put the rest on the bottom of your library in a random order." The cards are looked at (a smaller
/// library shows what it has, CR 609.3), up to one creature card of mana value 4 or less among them is chosen, and the
/// others still in the library go to the bottom in a random order (CR 401.4).
fn look_for_a_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let looked = top_of_library(g, you, 5);
    g.look_at(you, &looked);
    let creatures: Vec<ObjectId> = looked
        .iter()
        .copied()
        .filter(|&c| g.chars(c).is(CardType::Creature) && g.mana_value(c) <= 4)
        .collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::PutOntoBattlefield, "creature", looked.clone(), creatures, 1, true)?;
    if let Some(&creature) = chosen.first() {
        g.put_onto_battlefield(creature, you, false)?;
    }
    library::rest_to_bottom_in_random_order(g, you, &looked);
    Ok(())
}

/// "When another creature you control leaves the battlefield" (CR 603.6c, 603.10a), looked back on: a creature other than
/// Aang, under its controller's control as it last was on the battlefield, to whichever zone it went.
fn another_creature_you_control_leaves(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, .. }
            if *old != me.source
                && from.is(ZoneKind::Battlefield)
                && q.chars(*old).is(CardType::Creature)
                && q.obj(*old).controller == me.controller =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Transform Aang at the beginning of the next upkeep": a delayed triggered ability (CR 603.7), the next upkeep of
/// whichever player, made by Aang and about Aang. It transforms him only if he hasn't transformed since it was created
/// (CR 701.27f, the 2025-10-02 ruling), which the transform instruction itself asks.
fn transform_at_the_next_upkeep(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let o = *g.obj(item.object);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    g.delayed_trigger(DelayedWhen::Step { step: Step::Upkeep, on_turn_of: None }, item.object, item.controller, AbilityRef::delayed(card, 1), Target::None);
    Ok(())
}

/// "Land creatures you control" (CR 613.1f, layer 6): a creature under Aang's controller's control that is a land, as the
/// earlier layers made it (an earthbent land among them).
fn land_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Land) && statics::your_creatures(q, source, affected, so_far)
}

pub static BACK: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, land_creatures_you_control, statics::have_vigilance, "Land creatures you control have vigilance.")],
    abilities: &[triggered(BEGINNING_OF_YOUR_COMBAT, earthbend::effect::<2>, "At the beginning of combat on your turn, earthbend 2.")
        .targets(&[earthbend::TARGET])],
    ..oracle::AANG_DESTINED_SAVIOR
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, look_for_a_creature, "When Aang enters, look at the top five cards of your library. You may put a creature card with mana value 4 or less from among them onto the battlefield. Put the rest on the bottom of your library in a random order."),
        triggered(when_asked(EventMask::ZONE_CHANGE, another_creature_you_control_leaves), transform_at_the_next_upkeep, "When another creature you control leaves the battlefield, transform Aang at the beginning of the next upkeep.")
            .look_back(),
    ],
    // Earthbend's returning ability is the card's delayed ability 0 (`earthbend::RETURN`), the back face's earthbend
    // reading it off the card.
    delayed: &[earthbend::RETURN, delayed_trigger(effects::transform_self, "Transform Aang.")],
    back: Some(&BACK),
    ..oracle::AANG_AT_THE_CROSSROADS
};
