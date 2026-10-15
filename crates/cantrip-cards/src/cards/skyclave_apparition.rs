use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// A blue Illusion creature token (the generic definition: its size is set as it is made, X/X).
pub static ILLUSION: CardDef = CardDef {
    name: "Illusion",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Blue)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Illusion]),
    power: Some(0),
    toughness: Some(0),
    ..CardDef::BLANK
};

/// "Nonland, nontoken permanent you don't control with mana value 4 or less."
fn target_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if !q.obj(id).token && q.mana_value(id) <= 4)
}

/// "Exile up to one target ...", remembered as exiled with this creature (CR 607.2a).
fn exile_permanent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.exile_with(&[permanent], item.object, false);
    }
    Ok(())
}

/// "The exiled card's owner creates an X/X blue Illusion creature token, where X is the mana value of the exiled
/// card."
fn illusion_for_owner(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(&card) = g.exiled_with(item.object).first() else { return Ok(()) };
    let owner = g.obj(card).owner;
    let x = g.mana_value(card) as i16;
    let def = g.card_id(&ILLUSION);
    for token in g.create_tokens(def, owner, 1, false)? {
        g.add_effect(item.object, token, EffectKind::SetPt(x, x), Duration::Indefinite);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, exile up to one target nonland, nontoken permanent you don't control with mana value 4 or less.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            targets: &[TargetSpec { min: 0, max: 1, filter: target_filter }],
            effect: exile_permanent,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature leaves the battlefield, the exiled card's owner creates an X/X blue Illusion creature token, where X is the mana value of the exiled card.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_leaves,
            effect: illusion_for_owner,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::SKYCLAVE_APPARITION
};

pub static EXTRA: &[&CardDef] = &[&ILLUSION];
