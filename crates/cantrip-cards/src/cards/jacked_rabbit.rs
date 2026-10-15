use cantrip_api::*;
use cantrip_mechanics::ravenous;
use cantrip_oracle as oracle;

/// The 1/1 white Rabbit creature token its attack makes.
pub static RABBIT: CardDef = CardDef {
    name: "Rabbit",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Rabbit]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// Create a number of 1/1 white Rabbit creature tokens equal to this creature's power.
fn make_rabbits(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u32;
    if power > 0 {
        cantrip_mechanics::tokens::create(g, item.controller, &RABBIT, power)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(ravenous::enters),
    abilities: &[
        ravenous::draw_trigger("When Jacked Rabbit enters, if X is 5 or more, draw a card."),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Jacked Rabbit attacks, create a number of 1/1 white Rabbit creature tokens equal to its power.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::this_attacks,
            effect: make_rabbits,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::JACKED_RABBIT
};

pub static EXTRA: &[&CardDef] = &[&RABBIT];
