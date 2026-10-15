use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// The 1/1 colorless Robot artifact creature token. It shares its name with the other Robot token (Melded Moxite's).
pub static ROBOT: CardDef = CardDef {
    name: "Robot",
    has_cost: false,
    colors: Some(ColorSet::NONE),
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Robot]),
    power: Some(1),
    toughness: Some(1),
    shares_name: true,
    ..CardDef::BLANK
};

fn make_a_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ROBOT, 1)?;
    Ok(())
}

/// Creature tokens you control gain haste until end of turn: the set is fixed as the ability resolves, so a token made
/// afterwards is not reached (CR 611.2c).
fn tokens_gain_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let tokens: Vec<ObjectId> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| g.obj(id).token && g.chars(id).is(CardType::Creature))
        .collect();
    for id in tokens {
        g.add_effect(item.object, id, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you cast an artifact spell, create a 1/1 colorless Robot artifact creature token.",
            listens: EventMask::SPELL_CAST,
            matches: cantrip_mechanics::triggers::artifact_spell_you_cast,
            effect: make_a_robot,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{R}, {T}: Creature tokens you control gain haste until end of turn.",
            cost: Cost { mana: mana(0, 0, 0, 0, 1, 0), tap: true, ..Cost::NONE },
            effect: tokens_gain_haste,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::RAVENOUS_ROBOTS
};

pub static EXTRA: &[&CardDef] = &[&ROBOT];
