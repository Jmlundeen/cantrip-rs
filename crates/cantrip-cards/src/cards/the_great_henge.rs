use cantrip_api::*;
use cantrip_oracle as oracle;

/// This spell costs {X} less to cast, where X is the greatest power among creatures you control (CR 601.2f): generic
/// mana only; a creature with negative power reduces nothing, and with no creatures there is no reduction.
fn cheaper_by_greatest_power(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed) -> CostDelta {
    if !matches!(costed, Costed::Spell(_)) {
        return CostDelta::NONE;
    }
    let greatest = q
        .permanents_of(player)
        .into_iter()
        .map(|id| q.chars(id))
        .filter(|c| c.is(CardType::Creature))
        .map(|c| c.power.unwrap_or(0))
        .max()
        .unwrap_or(0)
        .max(0);
    CostDelta::less(greatest.min(255) as u8)
}

/// "{T}: Add {G}{G}. You gain 2 life." A mana ability: the life is gained as it resolves.
fn add_gg_and_gain(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::G, 2);
    g.gain_life(asker.controller, 2);
    Ok(())
}

/// "Whenever a nontoken creature you control enters": judged on the permanent as it entered (CR 603.6a). Each of several
/// entering at once triggers it separately, and "it" is carried as the subject.
fn nontoken_creature_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if to.is(ZoneKind::Battlefield) => {
            let entered = q.obj(*new);
            (is_live(q, *new)
                && q.is_on_battlefield(*new)
                && !entered.token
                && entered.controller == q.controller(source)
                && q.chars(*new).is(CardType::Creature))
            .then_some(Fired { subject: Target::Object(*new), data: [0; 2] })
        }
        _ => None,
    }
}

/// Put a +1/+1 counter on it and draw a card.
fn counter_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(it) = item.subject
        && is_live(g, it)
        && g.is_on_battlefield(it)
    {
        g.add_counters(it, CounterKind::PlusOne, 1);
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[cheaper_by_greatest_power],
    abilities: &[
        mana_ability(TAP, add_n(Mana::G, 2).by(add_gg_and_gain), "{T}: Add {G}{G}. You gain 2 life."),
        triggered(when(EventMask::ZONE_CHANGE, nontoken_creature_enters), counter_and_draw, "Whenever a nontoken creature you control enters, put a +1/+1 counter on it and draw a card."),
    ],
    ..oracle::THE_GREAT_HENGE
};
