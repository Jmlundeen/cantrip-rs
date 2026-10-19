use cantrip_api::*;
use cantrip_mechanics::effects::gain_lifelink_until_end_of_turn;
use cantrip_mechanics::history;
use cantrip_mechanics::triggers::EACH_END_STEP;
use cantrip_mechanics::{pump, tokens};
use cantrip_oracle as oracle;

/// The 4/4 white Angel creature token with flying and vigilance it makes.
pub static ANGEL: CardDef = CardDef {
    name: "Angel",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Angel]),
    power: Some(4),
    toughness: Some(4),
    keywords: Keywords::of(&[Keyword::Flying, Keyword::Vigilance]),
    ..CardDef::BLANK
};

/// "Create a 4/4 white Angel creature token with flying and vigilance."
fn create_angel(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ANGEL, 1)?;
    Ok(())
}

/// "Until end of turn, this creature gets +2/+2 and gains lifelink" (CR 611.2, 702.15).
fn pump_and_lifelink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 2, 2);
    gain_lifelink_until_end_of_turn(g, item.object, item.object);
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&ANGEL];

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(EACH_END_STEP, create_angel, "At the beginning of each end step, if you gained 5 or more life this turn, create a 4/4 white Angel creature token with flying and vigilance.")
            .condition(history::you_gained_five_or_more_life_this_turn),
        ability(cost("{3}{W}{W}{W}"), pump_and_lifelink, "{3}{W}{W}{W}: Until end of turn, this creature gets +2/+2 and gains lifelink."),
    ],
    ..oracle::RESPLENDENT_ANGEL
};
