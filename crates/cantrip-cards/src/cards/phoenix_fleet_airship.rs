use cantrip_api::*;
use cantrip_mechanics::{copy::create_token_copy_of_it, crew::crew, history::permanents_sacrificed_this_turn, triggers::YOUR_END_STEP};
use cantrip_oracle as oracle;

const NAME: &str = "Phoenix Fleet Airship";

/// "If you sacrificed a permanent this turn" (CR 603.4, 701.21a): checked as the end step begins and again as the ability
/// resolves.
fn sacrificed_a_permanent_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    permanents_sacrificed_this_turn(q, asker.controller) > 0
}

/// "Create a token that's a copy of this Vehicle" (CR 707.2, 111.4): its copiable values, or its last known ones if it has left
/// the battlefield by now (CR 608.2h).
fn copy_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_token_copy_of_it(g, item, |_, values| values)
}

/// "As long as you control eight or more permanents named Phoenix Fleet Airship" (CR 604.2, 201.2): this Vehicle, while its
/// controller controls at least eight permanents with that name as the earlier layers made them, itself included.
fn itself_with_eight_airships(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = q.state();
    let you = state.obj(source).controller;
    source == affected
        && state.battlefield().iter().filter(|&&id| state.obj(id).controller == you && q.chars(id).name == NAME).count() >= 8
}

/// "This Vehicle is an artifact creature" (CR 301.7, 613.1d, layer 4): a creature in addition to its other types.
fn artifact_creature(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Creature);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            YOUR_END_STEP,
            copy_this,
            "At the beginning of your end step, if you sacrificed a permanent this turn, create a token that's a copy of this Vehicle.",
        )
        .condition(sacrificed_a_permanent_this_turn),
        crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    statics: &[static_ability(
        Layer::Type,
        itself_with_eight_airships,
        artifact_creature,
        "As long as you control eight or more permanents named Phoenix Fleet Airship, this Vehicle is an artifact creature.",
    )],
    ..oracle::PHOENIX_FLEET_AIRSHIP
};
