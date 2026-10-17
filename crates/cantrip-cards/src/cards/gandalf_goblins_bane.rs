use cantrip_api::*;
use cantrip_mechanics::{effects, pump, triggers};
use cantrip_oracle as oracle;

/// "Look at the top two cards of your library and exile them face down. For as long as they remain exiled, you may play
/// them if you control a Wizard."
///
/// The cards are exiled face down (CR 406.3) and stay known to their controller, who looked at them; each is theirs to
/// play while it stays in exile (CR 611.2, 400.7), at the times the rules allow and paying its costs (CR 601.3, 305.1;
/// ruling 2026-06-29), whenever they control a Wizard as they would play it. "Wizard" is read through the spell's words
/// (CR 612).
fn flameshape(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let top: Vec<ObjectId> = g.state().zone(Zone::library(you)).iter().take(2).copied().collect();
    let exiled = g.exile_face_down(&top, None);
    g.look_at(you, &exiled);
    let wizard = g.words(item.object).subtype(Subtype::Wizard);
    for card in exiled {
        if g.obj(card).zone.is(ZoneKind::Exile) {
            g.allow_play(card, you, PlayGrant { lands: true, when: Some(PlayCondition::ControlsSubtype(wizard)), ..PlayGrant::CAST });
        }
    }
    Ok(())
}

/// The Adventure (CR 715): Flameshape.
pub static FLAMESHAPE: CardDef = CardDef { spell: spell(&[], flameshape), ..oracle::FLAMESHAPE };

/// "Whenever you cast a noncreature spell, Gandalf gets +1/+1 until end of turn and deals 1 damage to each opponent": the
/// pump only while Gandalf is still on the battlefield, the damage by Gandalf as it last was if it has left (CR 113.7a).
fn grow_and_burn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, 1);
    effects::damage_each_opponent(g, item, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, triggers::noncreature_spell_you_cast),
        grow_and_burn,
        "Whenever you cast a noncreature spell, Gandalf gets +1/+1 until end of turn and deals 1 damage to each opponent.",
    )],
    adventure: Some(&FLAMESHAPE),
    ..oracle::GANDALF_GOBLINS_BANE
};

pub static EXTRA: &[&CardDef] = &[&FLAMESHAPE];
