use cantrip_api::*;
use cantrip_mechanics::each_opponent_sacrifices;
use cantrip_mechanics::triggers::{self, ward_poison};
use cantrip_oracle as oracle;

/// "Whenever another creature you control with deathtouch dies" (CR 700.4, 603.10a): the shared "a creature you control dies"
/// (`triggers::creature_you_control_dies`) of a creature other than this one with deathtouch as it last was on the battlefield.
fn another_deathtouch_creature_you_control_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let fired = triggers::creature_you_control_dies(q, me, event)?;
    match event {
        Event::ZoneChange { old, .. } if *old != me.source && q.chars(*old).has(Keyword::Deathtouch) => Some(fired),
        _ => None,
    }
}

fn nontoken_creature(g: &dyn Query, id: ObjectId) -> bool {
    g.chars(id).is(CardType::Creature) && !g.obj(id).token
}

fn sacrifice_nontoken(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, nontoken_creature)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_poison::<5>("Ward—Get five poison counters."),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, another_deathtouch_creature_you_control_dies),
            sacrifice_nontoken,
            "Whenever another creature you control with deathtouch dies, each opponent sacrifices a nontoken creature of their choice.",
        ),
    ],
    ..oracle::THE_SERPENT_SOCIETY
};
