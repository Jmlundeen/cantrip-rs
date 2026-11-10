use cantrip_api::*;
use cantrip_mechanics::{effects, station, statics, targets, triggers};
use cantrip_oracle as oracle;

/// The 4+ striation, "Flying" with the 5/5 power/toughness box (CR 721.2b): while it has four or more charge counters it has
/// flying and is a creature with base power and toughness 5/5 in addition to its other types - an artifact creature, as its
/// reminder text says. One static ability, applied in each layer it touches (CR 613.6).
const FOUR_PLUS: &str = "4+ | Flying (5/5)";

/// "One or two target Klingons and/or Warriors you control" (CR 115.1, 109.2, 205.3m): a permanent on the battlefield under
/// the ability's controller's control with either creature type as the layers made it, a changeling included, whether or not
/// it is a creature. Each type is read
/// through the words of the text of the card that asks (CR 612.2). One slot takes one or two of them (CR 601.2c).
fn klingon_or_warrior_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let words = q.words(asker.source);
    targets::permanent_you_control_filter(q, asker, t)
        && matches!(t, Target::Object(id)
            if [Subtype::Klingon, Subtype::Warrior].iter().any(|&s| q.chars(id).has_subtype(words.subtype(s))))
}

/// "Whenever this Spacecraft enters or attacks, one or two target Klingons and/or Warriors you control gain flying until end
/// of turn." (CR 603.4, 611.2a, 702.9b): each target still legal as the ability resolves gains flying for the rest of the turn;
/// a target that has left the battlefield, or is no longer a Klingon or Warrior, gains nothing (CR 608.2b).
fn one_or_two_gain_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::targets_gain(g, item, Keyword::Flying)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS_OR_ATTACKS,
            one_or_two_gain_flying,
            "Whenever this Spacecraft enters or attacks, one or two target Klingons and/or Warriors you control gain flying until end of turn.",
        )
        .targets(&[target(klingon_or_warrior_you_control).between(1, 2)]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 4+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<4>, station::is_a_creature, FOUR_PLUS),
        static_ability(Layer::Ability, station::charged::<4>, statics::have_flying, FOUR_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<4>, statics::base_pt::<5, 5>, FOUR_PLUS).continues(),
    ],
    ..oracle::KTINGA_CLASS_BATTLE_CRUISER
};
