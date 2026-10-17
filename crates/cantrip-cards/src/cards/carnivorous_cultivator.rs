use cantrip_api::*;
use cantrip_mechanics::library::{search_to_graveyard, Find};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

/// "Search your library for a land card, put it into your graveyard, then shuffle." (CR 701.23.) A search for a quality
/// may find nothing (CR 701.23b), and the library is shuffled either way.
fn enroot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_graveyard(g, item.controller, item.object, Find::ONE, |q, c| q.chars(c).is(CardType::Land))?;
    Ok(())
}

/// Carnivorous Cultivator's prepare spell (CR 722): not a card, only ever a copy.
pub static ENROOT: CardDef = CardDef {
    name: "Enroot",
    cost: mana("{G}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], enroot),
    text: "Search your library for a land card, put it into your graveyard, then shuffle.",
    ..CardDef::BLANK
};

/// "Return target land card from your graveyard to your hand." (CR 400.7: a target that left the graveyard is illegal.)
fn return_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(land) = targets::object(g, item, 0) {
        g.return_to_hand(&[land]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, return_land, "Whenever this creature deals combat damage to a player, return target land card from your graveyard to your hand.")
        .targets(&[targets::LAND_CARD_IN_YOUR_GRAVEYARD])],
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&ENROOT),
    ..oracle::CARNIVOROUS_CULTIVATOR
};

pub static EXTRA: &[&CardDef] = &[&ENROOT];
