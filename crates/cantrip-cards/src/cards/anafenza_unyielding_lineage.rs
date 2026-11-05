use cantrip_api::*;
use cantrip_mechanics::{endure, triggers};
use cantrip_oracle as oracle;

/// "Whenever another nontoken creature you control dies" (CR 700.4, 603.10a): `another_creature_you_control_dies` for a creature that
/// was not a token (CR 111.7), looked back on as it last was on the battlefield.
fn another_nontoken_creature_you_control_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, .. } if q.obj(*old).token => None,
        _ => triggers::another_creature_you_control_dies(q, me, event),
    }
}

/// "Whenever another nontoken creature you control dies, Anafenza endures 2." (CR 701.63a, 603.10a): Anafenza endures even when
/// it died in the same event, and then only the Spirit token can be made.
fn endures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    endure::endure(g, item.object, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, another_nontoken_creature_you_control_dies),
        endures,
        "Whenever another nontoken creature you control dies, Anafenza endures 2. (Put two +1/+1 counters on it or create a 2/2 white Spirit creature token.)",
    )],
    ..oracle::ANAFENZA_UNYIELDING_LINEAGE
};

pub static EXTRA: &[&CardDef] = &[&endure::SPIRIT];
