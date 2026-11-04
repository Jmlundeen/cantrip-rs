use cantrip_api::*;
use cantrip_mechanics::{damage_target, statics, station, targets, triggers};
use cantrip_oracle as oracle;

const TEN_PLUS: &str = "10+ | Whenever you attack, Dawnsire deals 100 damage to up to one target creature or planeswalker.";
const TWENTY_PLUS: &str = "20+ | Flying";

/// "Dawnsire deals 100 damage to up to one target creature or planeswalker" (CR 120.3, 608.2b): with no target chosen nothing
/// is dealt damage; a target that is no longer legal is not dealt damage.
fn deal_100_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 100);
    Ok(())
}

/// The 10+ striation (CR 721.2a) holds a triggered ability, the card's `delayed` list; the 20+ striation makes the Spacecraft a
/// 20/20 artifact creature with flying, and its one layer-6 effect continues the layer-4 type change.
pub static CARD: CardDef = CardDef {
    abilities: &[station::ability(
        "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 20+.)",
    )],
    statics: &[
        static_ability(Layer::Ability, station::charged::<10>, station::has::<0>, TEN_PLUS),
        static_ability(Layer::Type, station::charged::<20>, station::is_a_creature, TWENTY_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, TWENTY_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<20, 20>, TWENTY_PLUS).continues(),
    ],
    delayed: &[triggered(
        triggers::YOU_ATTACK,
        deal_100_damage,
        "Whenever you attack, Dawnsire deals 100 damage to up to one target creature or planeswalker.",
    )
    .targets(&[up_to_one(targets::CREATURE_OR_PLANESWALKER.filter)])],
    ..oracle::DAWNSIRE_SUNSTAR_DREADNOUGHT
};
