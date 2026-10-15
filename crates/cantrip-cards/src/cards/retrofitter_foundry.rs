use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::tokens::SERVO;
use cantrip_oracle as oracle;

/// A 1/1 colorless Thopter artifact creature token with flying.
pub static THOPTER: CardDef = CardDef {
    name: "Thopter",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Thopter]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..CardDef::BLANK
};

/// A 4/4 colorless Construct artifact creature token.
pub static CONSTRUCT: CardDef = CardDef {
    name: "Construct",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Construct]),
    power: Some(4),
    toughness: Some(4),
    ..CardDef::BLANK
};

fn is_servo(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(Subtype::Servo)
}

fn is_thopter(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(Subtype::Thopter)
}

/// {3}: Untap this artifact.
fn untap_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.untap(item.object);
    }
    Ok(())
}

fn make_servo(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &SERVO, 1)?;
    Ok(())
}

fn make_thopter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &THOPTER, 1)?;
    Ok(())
}

fn make_construct(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &CONSTRUCT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{3}: Untap Retrofitter Foundry.",
            cost: Cost::mana(mana(3, 0, 0, 0, 0, 0)),
            effect: untap_this,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{2}, {T}: Create a 1/1 colorless Servo artifact creature token.",
            cost: Cost { mana: mana(2, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: make_servo,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, {T}, Sacrifice a Servo: Create a 1/1 colorless Thopter artifact creature token with flying.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, ..Cost::sacrifice(1, is_servo) },
            effect: make_thopter,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice a Thopter: Create a 4/4 colorless Construct artifact creature token.",
            cost: Cost { tap: true, ..Cost::sacrifice(1, is_thopter) },
            effect: make_construct,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::RETROFITTER_FOUNDRY
};

pub static EXTRA: &[&CardDef] = &[&SERVO, &THOPTER, &CONSTRUCT];
