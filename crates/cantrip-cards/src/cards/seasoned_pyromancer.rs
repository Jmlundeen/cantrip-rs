use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::{choose_from_hand, triggers::this_enters};
use cantrip_oracle as oracle;

/// The 1/1 red Elemental creature token it makes.
pub static ELEMENTAL: CardDef = CardDef {
    name: "Elemental",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Elemental]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// Discard two cards (all of a shorter hand) as one action, draw two, then an Elemental for each nonland card
/// discarded this way, created together (CR 701.9b, 101.4). A card's type is read before it moves (CR 400.7).
fn enter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let chosen = choose_from_hand(g, you, item.object, 2)?;
    let nonlands = chosen.iter().filter(|&&c| !g.chars(c).is(CardType::Land)).count();
    g.discard_together(&chosen);
    g.draw(you, 2);
    if nonlands > 0 {
        let elemental = g.card_id(&ELEMENTAL);
        let tokens: Vec<(CardId, PlayerId)> = (0..nonlands).map(|_| (elemental, you)).collect();
        g.create_tokens_together(&tokens)?;
    }
    Ok(())
}

fn two_elementals(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ELEMENTAL, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, discard two cards, then draw two cards. For each nonland card discarded this way, create a 1/1 red Elemental creature token.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: enter,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{R}{R}, Exile this card from your graveyard: Create two 1/1 red Elemental creature tokens.",
            cost: Cost { mana: mana(3, 0, 0, 0, 2, 0), exile_self: true, ..Cost::NONE },
            effect: two_elementals,
            from: ZoneKind::Graveyard,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::SEASONED_PYROMANCER
};

pub static EXTRA: &[&CardDef] = &[&ELEMENTAL];
