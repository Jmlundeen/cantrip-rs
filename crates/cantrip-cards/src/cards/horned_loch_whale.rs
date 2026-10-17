use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::ward;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("PutOnTop", "put your creature on top of your library?", "If you don't, it goes on the bottom", "Top", "Bottom");

/// "Target attacking creature you don't control" (CR 115.1, 506.4, 109.5): attacking when chosen and when the spell resolves.
fn attacking_creature_you_dont_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::ATTACKING_CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

/// "The owner of target attacking creature you don't control puts it on their choice of the top or bottom of their
/// library." (CR 401.4, 400.3) The owner, not the controller, chooses; yes is the top. With its target gone the spell does
/// not resolve (CR 608.2b).
fn lagoon_breach(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(creature).owner;
    let top = ask_may(g, owner, item.object, Purpose::Ask(&ASK))?;
    g.put_into_library(&[creature], if top { LibraryPlace::Top } else { LibraryPlace::Bottom });
    Ok(())
}

/// The Adventure (CR 715).
pub static LAGOON_BREACH: CardDef = CardDef {
    spell: spell(&[target(attacking_creature_you_dont_control)], lagoon_breach),
    ..oracle::LAGOON_BREACH
};

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::mana::enters_tapped_unless_your_turn),
    abilities: &[ward::<2>("Ward {2}")],
    adventure: Some(&LAGOON_BREACH),
    ..oracle::HORNED_LOCH_WHALE
};

pub static EXTRA: &[&CardDef] = &[&LAGOON_BREACH];
