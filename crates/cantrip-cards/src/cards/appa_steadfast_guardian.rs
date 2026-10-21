use cantrip_api::*;
use cantrip_mechanics::{airbend, triggers::{THIS_ENTERS, YOU_CAST_A_SPELL_FROM_EXILE}, targets, tokens};
use cantrip_oracle as oracle;

/// "Any number of other target nonland permanents you control."
fn other_nonland_permanent_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source && q.controller(id) == asker.controller)
        && (targets::NONLAND_PERMANENT.filter)(q, asker, t)
}

fn airbend_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    airbend(g, item);
    Ok(())
}

fn ally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ALLY, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, airbend_them, "When Appa enters, airbend any number of other target nonland permanents you control.")
            .targets(&[up_to(TargetSpec::ANY_NUMBER, other_nonland_permanent_you_control)]),
        triggered(YOU_CAST_A_SPELL_FROM_EXILE, ally, "Whenever you cast a spell from exile, create a 1/1 white Ally creature token."),
    ],
    ..oracle::APPA_STEADFAST_GUARDIAN
};

pub static EXTRA: &[&CardDef] = &[&tokens::ALLY];
