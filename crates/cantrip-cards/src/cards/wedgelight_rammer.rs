use cantrip_api::*;
use cantrip_mechanics::{statics, station, tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

const NINE_PLUS: &str = "9+ | Flying, first strike";

/// "Create a 2/2 colorless Robot artifact creature token" (CR 111.1): the ability's controller creates it.
fn robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ROBOT_2_2, 1)?;
    Ok(())
}

/// The 9+ striation's "Flying, first strike" (CR 702.9, 702.7, 721.2a): both in the one layer-6 effect.
fn have_flying_and_first_strike(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying).with(Keyword::FirstStrike);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, robot, "When this Spacecraft enters, create a 2/2 colorless Robot artifact creature token."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 9+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<9>, station::is_a_creature, NINE_PLUS),
        static_ability(Layer::Ability, station::charged::<9>, have_flying_and_first_strike, NINE_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<9>, statics::base_pt::<3, 4>, NINE_PLUS).continues(),
    ],
    ..oracle::WEDGELIGHT_RAMMER
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROBOT_2_2];
