use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_oracle as oracle;

fn draw_ability(q: &dyn Query, source: ObjectId) -> AbilityRef {
    AbilityRef { card: q.chars(source).card, index: 0 }
}

/// "Whenever you put one or more +1/+1 counters on a creature you control" (CR 122.6a, 603.2c): the player who put
/// them is the event's, so an opponent's effect putting counters on your creature does not trigger it, and a creature
/// entering under your control with counters does. Not at all once its controller has drawn with it this turn
/// (CR 603.2h). The counters put at once carry as the number to draw.
fn you_put_plus_one_counters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, kind: CounterKind::PlusOne, amount, by }
            if *amount > 0
                && *by == Some(q.controller(source))
                && q.controller(*object) == q.controller(source)
                && q.chars(*object).is(CardType::Creature)
                && !q.history().done(source, draw_ability(q, source)) =>
        {
            Some(Fired { subject: Target::Object(*object), data: [*amount as u32, 0] })
        }
        _ => None,
    }
}

/// "You may draw that many cards. Do this only once each turn." Asked only while its controller has not drawn with
/// this permanent this turn, and spent only by a yes.
fn may_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.data[0] == 0 || g.history().done(item.object, item.ability) {
        return Ok(());
    }
    if !ask_may(g, item.controller, item.object, Purpose::Optional)? {
        return Ok(());
    }
    g.note_done(item.object, item.ability);
    g.draw(item.controller, item.data[0]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::COUNTERS, you_put_plus_one_counters), may_draw, "Whenever you put one or more +1/+1 counters on a creature you control, you may draw that many cards. Do this only once each turn.")],
    ..oracle::TERRASYMBIOSIS
};
