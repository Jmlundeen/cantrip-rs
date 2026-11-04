use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever another nontoken creature you control dies" (CR 700.4, 603.10a): another creature you controlled as it died, one that
/// was not a token (CR 111.7), looked back on as it last was on the battlefield.
fn another_nontoken_creature_you_control_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, .. } if q.obj(*old).token => None,
        _ => triggers::another_creature_you_control_dies(q, me, event),
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, another_nontoken_creature_you_control_dies),
        effects::draw_a_card_and_lose_1_life,
        "Whenever another nontoken creature you control dies, you draw a card and lose 1 life.",
    )],
    ..oracle::AGENT_VENOM
};
