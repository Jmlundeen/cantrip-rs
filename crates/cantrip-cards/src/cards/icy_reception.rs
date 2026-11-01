use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature or legendary spell" (CR 115.1): a spell on the stack that is a creature spell or a legendary spell.
fn creature_or_legendary_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if { let c = q.chars(id); c.is(CardType::Creature) || c.supertypes.legendary })
}

static MODAL: ModalDef = modes(&[
    mode(
        effects::counter_first_target_unless_pays::<3>,
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
