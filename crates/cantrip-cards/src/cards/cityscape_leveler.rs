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

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::CAST_THIS_SPELL, level, "When you cast this spell, destroy up to one target nonland permanent. Its controller creates a tapped Powerstone token.")
            .targets(&[up_to_one(targets::NONLAND_PERMANENT.filter)]),
        triggered(triggers::THIS_ATTACKS, level, "Whenever this creature attacks, destroy up to one target nonland permanent. Its controller creates a tapped Powerstone token.")
            .targets(&[up_to_one(targets::NONLAND_PERMANENT.filter)]),
        unearth::ability(mana("{8}"), "Unearth {8}"),
    ],
    delayed: &[unearth::EXILE],
    ..oracle::CITYSCAPE_LEVELER
};

pub static EXTRA: &[&CardDef] = &[&POWERSTONE];
