use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever a token you control enters": any token, of any type, entering under its controller's control (CR 111.1,
/// 603.6a), Belladonna's own copy-token entry included.
fn token_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.obj(*new).token && q.controller(*new) == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// What the trigger does depends on how many times this ability of this Belladonna has resolved this turn before
/// (CR 608.2m): this turn's history counts them. First, gain 1 life. Second, draw a card. Third, a +1/+1 counter on each
/// creature you control. A fourth and later resolution does nothing.
fn token_enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    match g.history().resolutions(item.object, item.ability) {
        0 => effects::gain_life(g, item, 1)?,
        1 => effects::draw(g, item, 1)?,
        2 => {
            effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
        }
        _ => {}
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, token_you_control_enters), token_enters, "Whenever a token you control enters, you gain 1 life if this is the first time this ability has resolved this turn. If it's the second time, draw a card. If it's the third time, put a +1/+1 counter on each creature you control.")],
    ..oracle::BELLADONNA_TOOK
};
