use cantrip_api::*;
use cantrip_mechanics::{choose, select, statics, station, triggers};
use cantrip_oracle as oracle;

const EIGHT_PLUS: &str = "8+ | Flying";

/// "{T}: Each opponent exiles a card from their graveyard. You may exile a card from your graveyard." (CR 608.2d, 406.2):
/// each opponent chooses the card of their own graveyard to exile, asked even when it is their only one and of no empty
/// graveyard; then this ability's controller may exile one from their own, choosing none to decline. Every card exiled
/// this way is linked to the ability's source (CR 607.2a), for its last ability's return.
fn exiles_from_each_graveyard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for opponent in g.state().opponents(item.controller) {
        let graveyard = g.state().zone(Zone::graveyard(opponent)).to_vec();
        let chosen = choose::select(g, opponent, item.object, Purpose::Exile, graveyard, select::cards(select::exactly(1)))?;
        if !chosen.is_empty() {
            g.exile_with(&chosen, g.linked(item), false);
        }
    }
    let graveyard = g.state().zone(Zone::graveyard(item.controller)).to_vec();
    let exiled = choose::select(g, item.controller, item.object, Purpose::Exile, graveyard, select::cards(select::up_to(1)))?;
    if !exiled.is_empty() {
        g.exile_with(&exiled, g.linked(item), false);
    }
    Ok(())
}

/// "Whenever S.S. Botany Bay deals combat damage to a player, return each creature card you own exiled with it to the
/// battlefield." (CR 510.2, 607.2a, 110.2a): only the creature cards the exile linked to this ability's source, and only
/// ones its controller owns - a creature card of theirs an opponent exiled with it comes back too - each under its
/// owner's control, together (CR 101.4).
fn return_own_creatures_exiled_with_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<(ObjectId, PlayerId)> = g
        .exiled_with(g.linked(item))
        .into_iter()
        .filter(|&card| g.obj(card).owner == item.controller && g.chars(card).is(CardType::Creature))
        .map(|card| (card, g.obj(card).owner))
        .collect();
    if !cards.is_empty() {
        g.put_all_onto_battlefield(&cards)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, exiles_from_each_graveyard, "{T}: Each opponent exiles a card from their graveyard. You may exile a card from your graveyard."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)"),
        triggered(
            triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_EVENT,
            return_own_creatures_exiled_with_it,
            "Whenever S.S. Botany Bay deals combat damage to a player, return each creature card you own exiled with it to the battlefield.",
        ),
    ],
    statics: &[
        // "{8+}[abilities][P/T]" (CR 721.2a-b): while it has eight or more charge counters it is an artifact creature
        // with base power and toughness 3/5 in addition to its other types, and has the striation's static ability.
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<3, 5>, EIGHT_PLUS).continues(),
    ],
    ..oracle::S_S_BOTANY_BAY
};
