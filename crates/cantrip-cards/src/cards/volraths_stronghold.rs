use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature card from your graveyard" (CR 115.1).
fn creature_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature])
}

/// Put the target on top of its owner's library (CR 401.4), if it is still a legal target.
fn put_on_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_into_library(&[card], LibraryPlace::Top);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{1}{B}, {T}"), put_on_top, "{1}{B}, {T}: Put target creature card from your graveyard on top of your library.")
            .targets(&[target(creature_card_in_your_graveyard)]),
    ],
    ..oracle::VOLRATHS_STRONGHOLD
};
