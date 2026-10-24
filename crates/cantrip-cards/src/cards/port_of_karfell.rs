use cantrip_api::*;
use cantrip_mechanics::{choose, mana, select};
use cantrip_oracle as oracle;

/// "Mill four cards, then return a creature card from your graveyard to the battlefield tapped." (CR 701.17, 608.2c): the
/// graveyard is read after the mill, so a creature just milled may be chosen. The card does not target; the return is not
/// optional, so a creature card is chosen whenever the graveyard holds one.
fn mill_and_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 4);
    g.end_sentence();
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    if let Some(&card) = choose::select(g, you, item.object, Purpose::PutOntoBattlefield, graveyard, select::creature_cards(select::as_many_as_can(1)))?.first() {
        g.put_onto_battlefield(card, you, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add(Mana::U), "{T}: Add {U}."),
        ability(
            cost("{3}{U}{B}{B}").and(TAP).and(SACRIFICE_THIS),
            mill_and_return,
            "{3}{U}{B}{B}, {T}, Sacrifice this land: Mill four cards, then return a creature card from your graveyard to the battlefield tapped.",
        ),
    ],
    ..oracle::PORT_OF_KARFELL
};
