use cantrip_api::*;
use cantrip_mechanics::{mana, station};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::U), "{T}: Add {U}."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Planet. Station only as a sorcery.)"),
    ],
    // "This land enters tapped" (CR 614.1d).
    enters: Some(mana::enters_tapped),
    // The 12+ striation (CR 721.2a): a mana ability (CR 605.1a) Uthros has while it has twelve or more charge counters.
    delayed: &[mana_ability(cost("{U}, {T}"), add_by(mana::add_blue_for_each_artifact), "{U}, {T}: Add {U} for each artifact you control.")],
    statics: &[static_ability(Layer::Ability, station::charged::<12>, station::has::<0>, "12+ | {U}, {T}: Add {U} for each artifact you control.")],
    ..oracle::UTHROS_TITANIC_GODCORE
};
