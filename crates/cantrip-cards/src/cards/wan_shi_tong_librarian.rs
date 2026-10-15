use cantrip_api::*;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "Put X +1/+1 counters on him. Then draw half X cards, rounded down." X is the one the spell was cast with
/// (CR 107.3m), which the permanent keeps. The counters go on first (CR 608.2c) and only on him if he is still there;
/// the draw happens either way.
fn enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g.obj(item.object).x as i16;
    if x > 0 && g.is_on_battlefield(item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, x);
    }
    if x / 2 > 0 {
        g.draw(item.controller, (x / 2) as u32);
    }
    Ok(())
}

/// "Whenever an opponent searches their library" (CR 701.23): whether or not the search finds anything.
fn opponent_searches(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Searched { player } if *player != q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn grow_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Wan Shi Tong enters, put X +1/+1 counters on him. Then draw half X cards, rounded down.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: enters,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever an opponent searches their library, put a +1/+1 counter on Wan Shi Tong and draw a card.",
            listens: EventMask::SEARCHED,
            matches: opponent_searches,
            effect: grow_and_draw,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::WAN_SHI_TONG_LIBRARIAN
};
