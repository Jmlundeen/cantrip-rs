use cantrip_api::*;
use cantrip_mechanics::{effects, counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// "Target creature or legendary spell" (CR 115.1): a spell on the stack that is a creature spell or a legendary spell.
fn creature_or_legendary_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if { let c = q.chars(id); c.is(CardType::Creature) || c.supertypes.legendary })
}

/// "Counter target creature or legendary spell unless its controller pays {3}" (CR 118.12, 701.6): the spell's controller
/// is asked, and may make mana for it while asked; nothing if the target spell is gone (CR 608.2b).
fn counter_unless_paid(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{3}"))?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(
        counter_unless_paid,
        "Counter target creature or legendary spell unless its controller pays {3}.",
    )
    .targets(&[target(creature_or_legendary_spell)]),
    mode(effects::target_creature_gets::<-5, 0>, "Target creature gets -5/-0 until end of turn.").targets(&[targets::CREATURE]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::ICY_RECEPTION
};
