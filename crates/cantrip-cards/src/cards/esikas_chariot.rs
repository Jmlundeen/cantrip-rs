use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::{crew::crew, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// The 2/2 green Cat creature token.
pub static CAT: CardDef = CardDef {
    name: "Cat",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Cat]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

/// "Create two 2/2 green Cat creature tokens", together.
fn two_cats(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &CAT, 2)?;
    Ok(())
}

/// "Target token you control" (CR 115.1).
fn token_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::PERMANENT.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.obj(id).token && q.controller(id) == asker.controller)
}

/// "Create a token that's a copy of target token you control."
fn copy_token(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(token) = targets::object(g, item, 0) {
        g.create_token_copies(token, item.controller, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, two_cats, "When Esika's Chariot enters, create two 2/2 green Cat creature tokens."),
        triggered(cantrip_mechanics::triggers::THIS_ATTACKS, copy_token, "Whenever Esika's Chariot attacks, create a token that's a copy of target token you control.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: token_you_control, label: "" }]),
        crew(4, "Crew 4 (Tap any number of creatures you control with total power 4 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::ESIKAS_CHARIOT
};

pub static EXTRA: &[&CardDef] = &[&CAT];
