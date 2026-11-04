use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, statics, station};
use cantrip_oracle as oracle;

const EIGHT_PLUS: &str = "8+ | Flying, trample";

/// The 8+ striation (CR 721.2a): flying and trample, in the one layer-6 effect that continues the type change.
fn have_flying_and_trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying).with(Keyword::Trample);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            effects::plus_one_counter_on_each_creature_you_control,
            "When this Spacecraft enters, put a +1/+1 counter on each creature you control.",
        ),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, statics::itself, have_flying_and_trample, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<5, 4>, EIGHT_PLUS).continues(),
    ],
    ..oracle::ATMOSPHERIC_GREENHOUSE
};
