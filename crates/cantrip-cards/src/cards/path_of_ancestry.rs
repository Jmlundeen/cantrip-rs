use cantrip_api::*;
use cantrip_mechanics::{commander, effects, mana, statics};
use cantrip_oracle as oracle;

/// Whether the creature spell shares a creature type with any commander its caster owns, wherever that commander is
/// (CR 903.3e), read as the spell is cast (the 2023 ruling: not as the mana was made). Only creature types are
/// shared (CR 205.3m): a commander with none shares none.
fn shares_a_commander_type(q: &dyn Query, me: &Asker, spell: ObjectId) -> bool {
    let spell_types = q.chars(spell).subtypes;
    let zones = [
        Zone::COMMAND,
        Zone::BATTLEFIELD,
        Zone::STACK,
        Zone::PHASED_OUT,
        Zone::GONE,
        Zone::library(me.controller),
        Zone::hand(me.controller),
        Zone::graveyard(me.controller),
        Zone::exile(me.controller),
        Zone::sideboard(me.controller),
    ];
    zones
        .iter()
        .flat_map(|&zone| q.state().zone(zone).iter())
        .copied()
        .any(|id| {
            q.obj(id).commander
                && q.obj(id).owner == me.controller
                && statics::shares_creature_type(spell_types, q.chars(id).subtypes)
        })
}

/// "When that mana is spent to cast a creature spell that shares a creature type with your commander" (CR 603.7a,
/// 106.3): a delayed ability the mana ability creates, riding the mana it produced (`Adds::when_spent`), so it triggers
/// once for each mana of it spent - wherever Path of Ancestry has gone by then - and the core counts which mana was. The
/// spell is a creature as it is on the stack; "your" commander is that of the player who activated the ability, who
/// spent the mana (CR 603.7e).
fn spent_on_a_creature_sharing_a_commander_type(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match *event {
        Event::SpellCast { spell, .. } if q.chars(spell).is(CardType::Creature) && shares_a_commander_type(q, me, spell) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_any_color(1).when_spent(0),
        "{T}: Add one mana of any color in your commander's color identity. When that mana is spent to cast a creature spell that shares a creature type with your commander, scry 1.",
    )
    .colors_from(commander::colors)],
    delayed: &[triggered(
        when_asked(EventMask::SPELL_CAST, spent_on_a_creature_sharing_a_commander_type),
        |g, item| effects::scry(g, item, 1),
        "When that mana is spent to cast a creature spell that shares a creature type with your commander, scry 1.",
    )],
    enters: Some(mana::enters_tapped),
    ..oracle::PATH_OF_ANCESTRY
};
