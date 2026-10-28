use cantrip_api::*;
use cantrip_mechanics::effects::may_draw;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Draw", "draw that many cards?", "One for each +1/+1 counter put  ·  only once each turn", "Draw", "Don't draw");

fn draw_ability(q: &dyn Query, source: ObjectId) -> AbilityRef {
    AbilityRef { card: q.chars(source).card, index: 0 }
}

/// "Whenever you put one or more +1/+1 counters on a creature you control" (CR 122.6a, 603.2c): the player who put
/// them is the event's, so an opponent's effect putting counters on your creature does not trigger it, and a creature
/// entering under your control with counters does. Not at all once its controller has drawn with it this turn
/// (CR 603.2h). The counters put at once carry as the number to draw.
fn you_put_plus_one_counters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, kind: CounterKind::PlusOne, amount, by, .. }
            if *amount > 0
                && *by == q.controller(source)
                && q.controller(*object) == q.controller(source)
                && q.chars(*object).is(CardType::Creature)
                && !q.history().done(source, draw_ability(q, source), q.controller(source)) =>
        {
            Some(Fired { subject: Target::Object(*object), data: [*amount as u32, 0] })
        }
        _ => None,
    }
}

/// "You may draw that many cards. Do this only once each turn." Asked only while its controller has not drawn with
/// this permanent this turn, and spent only by a yes; not offered a player who can't draw that many (CR 121.3, 121.2b).
fn draw_that_many(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.data[0] == 0 || g.history().done(item.object, item.ability, item.controller) {
        return Ok(());
    }
    if may_draw(g, item.controller, item.object, item.data[0], &ASK)? {
        g.note_done(item.object, item.ability, item.controller);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::COUNTERS, you_put_plus_one_counters), draw_that_many, "Whenever you put one or more +1/+1 counters on a creature you control, you may draw that many cards. Do this only once each turn.")],
    ..oracle::TERRASYMBIOSIS
};
