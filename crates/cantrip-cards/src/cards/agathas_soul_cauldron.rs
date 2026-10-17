use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "You may spend mana as though it were mana of any color to activate abilities of creatures you control" (CR 609.4b).
fn creature_you_control(q: &dyn Query, asker: &Asker, spending: &Spending) -> bool {
    spending.player == asker.controller
        && spending.activating
        && spending.source.is_some_and(|source| {
            q.controller(source) == asker.controller && q.is_on_battlefield(source) && q.chars(source).is(CardType::Creature)
        })
}

/// The creature cards exiled with this permanent (CR 607.2a), in exile order.
fn exiled_creature_cards(view: &dyn LayerQuery, by: ObjectId) -> Vec<ObjectId> {
    let s = view.state();
    s.seats()
        .flat_map(|p| s.zone(Zone::exile(p)).iter().copied())
        .filter(|&id| s.obj(id).exiled.is_some_and(|l| l.by == by) && view.cards().get(s.obj(id).card).is(CardType::Creature))
        .collect()
}

/// "Creatures you control with +1/+1 counters on them" (CR 613.1f).
fn creature_with_counters(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = view.state();
    so_far.is(CardType::Creature)
        && s.obj(affected).controller == s.obj(source).controller
        && s.obj(affected).counters(CounterKind::PlusOne) > 0
}

/// "... have all activated abilities of all creature cards exiled with Agatha's Soul Cauldron": each is the creature's own
/// ability as written, so the creature is its source and "this creature" means it (CR 113.7, 613.1f).
/// The same ability twice is two abilities (CR 602.5c): a Rootwalla that exiled itself has its pump twice.
fn grant_abilities(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let cards: Vec<CardId> = exiled_creature_cards(view, source).into_iter().map(|c| view.state().obj(c).card).collect();
    cantrip_mechanics::statics::grant_activated_abilities_of(view, &cards, chars);
}

/// "{T}: Exile target card from a graveyard. When a creature card is exiled this way, put a +1/+1 counter on target
/// creature you control." The card is exiled with the Cauldron (CR 607.2a); the counter is a reflexive trigger (CR 603.12)
/// that exists only if a creature card really went to exile.
fn exile_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let creature = g.chars(card).is(CardType::Creature);
    let moved = g.exile_with(&[card], item.object, false);
    if creature && moved.first().is_some_and(|&c| g.obj(c).zone.is(ZoneKind::Exile)) {
        let me = g.obj(item.object).card;
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(me, 0), Target::None);
    }
    Ok(())
}

fn put_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, exile_card, "{T}: Exile target card from a graveyard. When a creature card is exiled this way, put a +1/+1 counter on target creature you control.")
        .targets(&[targets::CARD_IN_A_GRAVEYARD])],
    delayed: &[delayed_trigger(put_counter, "When a creature card is exiled this way, put a +1/+1 counter on target creature you control.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    statics: &[static_ability(Layer::Ability, creature_with_counters, grant_abilities, "Creatures you control with +1/+1 counters on them have all activated abilities of all creature cards exiled with Agatha's Soul Cauldron.")],
    spend_any_color: &[creature_you_control],
    ..oracle::AGATHAS_SOUL_CAULDRON
};
