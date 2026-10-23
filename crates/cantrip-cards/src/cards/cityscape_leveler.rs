use cantrip_api::*;
use cantrip_mechanics::tokens::POWERSTONE;
use cantrip_mechanics::{destroy_target, targets, triggers, unearth};
use cantrip_oracle as oracle;

/// "Destroy up to one target nonland permanent. Its controller creates a tapped Powerstone token." (CR 701.8, 111.10h):
/// choosing none does nothing, and a target that has become illegal makes the ability do nothing (CR 608.2b). The
/// controller is read before the destruction: an indestructible permanent survives and its controller still creates the
/// token, since the second sentence does not depend on the first.
fn level(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(target);
    destroy_target(g, item, 0);
    g.end_sentence();
    cantrip_mechanics::tokens::create_tapped_powerstone(g, controller)
}

/// "When you cast this spell and whenever this creature attacks": one ability with two trigger conditions (CR 603.2), the
/// first functioning from the stack and the second from the battlefield (CR 113.6k).
fn cast_or_attacks(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    triggers::cast_this_spell(q, source, event).or_else(|| triggers::this_attacks(q, source, event))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when(EventMask(EventMask::SPELL_CAST.0 | EventMask::ATTACKS.0), cast_or_attacks)
                .functions_in(ZoneKinds(ON_THE_STACK.0 | ON_THE_BATTLEFIELD.0)),
            level,
            "When you cast this spell and whenever this creature attacks, destroy up to one target nonland permanent. Its controller creates a tapped Powerstone token.",
        )
        .targets(&[up_to_one(targets::NONLAND_PERMANENT.filter)]),
        unearth::ability(mana("{8}"), "Unearth {8}"),
    ],
    delayed: &[unearth::EXILE],
    ..oracle::CITYSCAPE_LEVELER
};

pub static EXTRA: &[&CardDef] = &[&POWERSTONE];
