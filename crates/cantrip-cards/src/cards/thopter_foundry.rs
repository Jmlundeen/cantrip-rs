use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// A 1/1 blue Thopter artifact creature token with flying.
pub static THOPTER: CardDef = CardDef {
    name: "Thopter",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Blue)),
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Thopter]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..CardDef::BLANK
};

/// "Sacrifice a nontoken artifact": any artifact its controller controls that is not a token, the Foundry itself
/// among them.
fn nontoken_artifact(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    !q.obj(id).token && q.chars(id).is(CardType::Artifact)
}

fn make_thopter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &THOPTER, 1)?;
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&THOPTER];

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}").and(sacrifice(1, nontoken_artifact)), make_thopter, "{1}, Sacrifice a nontoken artifact: Create a 1/1 blue Thopter artifact creature token with flying. You gain 1 life.")],
    ..oracle::THOPTER_FOUNDRY
};
