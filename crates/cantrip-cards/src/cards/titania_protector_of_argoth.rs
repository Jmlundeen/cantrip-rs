use cantrip_api::*;
use cantrip_mechanics::{targets, tokens, triggers::this_enters};
use cantrip_oracle as oracle;

/// A 5/3 green Elemental creature token.
pub static ELEMENTAL: CardDef = CardDef {
    name: "Elemental",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Elemental]),
    power: Some(5),
    toughness: Some(3),
    ..CardDef::BLANK
};

fn land_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Land])
}

/// "Return target land card from your graveyard to the battlefield", if it is still a legal target.
fn return_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(land) = targets::object(g, item, 0) {
        g.put_onto_battlefield(land, item.controller, false)?;
    }
    Ok(())
}

/// "Whenever a land you control is put into a graveyard from the battlefield" (CR 700.4): judged on the board as it
/// was before it left, so the land's controller is the one it had there.
fn land_you_control_goes_to_graveyard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.chars(*old).is(CardType::Land)
                && q.controller(*old) == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn create_elemental(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ELEMENTAL, 1)?;
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&ELEMENTAL];

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Titania, Protector of Argoth enters, return target land card from your graveyard to the battlefield.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[TargetSpec { min: 1, max: 1, filter: land_card_in_your_graveyard }],
            effect: return_land,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever a land you control is put into a graveyard from the battlefield, create a 5/3 green Elemental creature token.",
            listens: EventMask::ZONE_CHANGE,
            matches: land_you_control_goes_to_graveyard,
            effect: create_elemental,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::TITANIA_PROTECTOR_OF_ARGOTH
};
