use cantrip_api::*;
use cantrip_mechanics::station;
use cantrip_oracle as oracle;

/// "Add X mana of any one color, where X is the number of charge counters on The Eternity Elevator" (CR 106.3, 605.1a): counted as the
/// ability resolves, all of the one color named as it was activated.
fn add_for_each_charge_counter(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    if let Some(color) = color {
        let counters = g.obj(asker.source).counters(CounterKind::Charge);
        g.add_mana(asker.controller, color.mana(), counters as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 3), "{T}: Add {C}{C}{C}."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery.)"),
    ],
    // The 20+ striation (CR 721.2a): a mana ability (CR 605.1a) the Spacecraft has while it has twenty or more charge counters. It
    // never becomes a creature: the striation has no power and toughness.
    delayed: &[mana_ability(TAP, add_any_color(1).by(add_for_each_charge_counter), "{T}: Add X mana of any one color, where X is the number of charge counters on The Eternity Elevator.")],
    statics: &[static_ability(
        Layer::Ability,
        station::charged::<20>,
        station::has::<0>,
        "20+ | {T}: Add X mana of any one color, where X is the number of charge counters on The Eternity Elevator.",
    )],
    ..oracle::THE_ETERNITY_ELEVATOR
};
