use cantrip_api::*;
use cantrip_mechanics::{statics, station, targets, triggers};
use cantrip_oracle as oracle;

fn target_creature_mv_four_or_less(q: &dyn Query, asker: &Asker, target: Target) -> bool {
    targets::card_in_your_graveyard_with_mana_value_at_most(
        q,
        asker,
        target,
        &[CardType::Creature],
        4,
    )
}

const TEN_PLUS: &str = "10+ | Flying";

/// "Whenever U.S.S. Reliant enters or deals combat damage to a player, return target creature card with mana value 4 or
/// less from your graveyard to the battlefield with a finality counter on it." The core target helper removes a target
/// that is illegal on resolution (CR 608.2b); the bulk put is a no-op for an empty set and makes each legal target enter
/// with its finality counter (CR 122.6, 614.1c).
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let entering: Vec<(ObjectId, PlayerId, bool)> = targets::legal_objects(g, item)
        .into_iter()
        .map(|card| (card, item.controller, false))
        .collect();
    g.put_all_onto_battlefield_with_counters(
        &entering,
        &[(CounterKind::Finality, 1)],
    )?;
    Ok(())
}
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, reanimate, "Whenever U.S.S. Reliant enters or deals combat damage to a player, return target creature card with mana value 4 or less from your graveyard to the battlefield with a finality counter on it.").targets(&[target(target_creature_mv_four_or_less)]),
        triggered(triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER, reanimate, "Whenever U.S.S. Reliant enters or deals combat damage to a player, return target creature card with mana value 4 or less from your graveyard to the battlefield with a finality counter on it.").targets(&[target(target_creature_mv_four_or_less)]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 10+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<10>, station::is_a_creature, TEN_PLUS),
        static_ability(Layer::Ability, station::charged::<10>, statics::have_flying, TEN_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<10>, statics::base_pt::<4, 3>, TEN_PLUS).continues(),
    ],
    ..oracle::U_S_S_RELIANT_FROM_HELLS_HEART
};
