use cantrip_api::*;
use cantrip_mechanics::{choose, effects, select, statics, targets};
use cantrip_oracle as oracle;

/// "-2: Mill three cards, then you may return a creature card with mana value 2 or less from your graveyard to the
/// battlefield." (CR 701.17a, 608.2c): the creature card is chosen from the graveyard as it is after the mill, so a card just
/// milled can come back.
fn mill_three_cards_return_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill_together(&[(you, 3)]);
    g.end_sentence();
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let rule = select::creature_cards(select::up_to(1)).with(select::mana_value_at_most(2));
    let chosen = choose::select(g, you, item.object, Purpose::PutOntoBattlefield, graveyard, rule)?;
    let entering: Vec<(ObjectId, PlayerId)> = chosen.into_iter().map(|card| (card, you)).collect();
    g.put_all_onto_battlefield(&entering)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), effects::untap_targets, "+1: Untap up to one target creature.")
            .targets(&[targets::CREATURE.between(0, 1)])
            .sorcery_speed(),
        ability(
            loyalty(-2),
            mill_three_cards_return_creature,
            "\u{2212}2: Mill three cards, then you may return a creature card with mana value 2 or less from your graveyard to the battlefield.",
        )
        .sorcery_speed(),
    ],
    activate_as_though_haste: &[statics::creature_you_control],
    ..oracle::TYVAR_JUBILANT_BRAWLER
};
