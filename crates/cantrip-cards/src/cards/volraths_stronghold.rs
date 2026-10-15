use cantrip_api::*;
use cantrip_mechanics::{mana, targets};
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
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{B}, {T}: Put target creature card from your graveyard on top of your library.",
            cost: Cost { mana: mana(1, 0, 0, 1, 0, 0), tap: true, ..Cost::NONE },
            targets: &[TargetSpec { min: 1, max: 1, filter: creature_card_in_your_graveyard }],
            effect: put_on_top,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::VOLRATHS_STRONGHOLD
};
