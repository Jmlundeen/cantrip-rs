use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::{delirium, mana};
use cantrip_oracle as oracle;

/// "Activate only if there are four or more card types among cards in your graveyard" (CR 602.5b).
fn has_delirium(q: &dyn Query, asker: &Asker) -> bool {
    delirium(q, asker.controller)
}

/// "Target permanent card in your graveyard" (CR 115.1): an artifact, battle, creature, enchantment, land or planeswalker card.
fn permanent_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(
        q,
        asker,
        t,
        &[CardType::Artifact, CardType::Battle, CardType::Creature, CardType::Enchantment, CardType::Land, CardType::Planeswalker],
    )
}

/// This land becomes a copy of target permanent card in your graveyard until end of turn (CR 707.2): still the same object, so
/// it stays tapped or untapped.
fn become_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        let values = g.copiable_values(card);
        g.become_copy(item.object, item.object, values);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_you_control::<{ cantrip_mechanics::lands::sub(Subtype::Forest) }>),
    abilities: &[
        mana::tap_for(mana::add_g, Mana::G, "{T}: Add {G}."),
        AbilityDef::Activated(ActivatedDef {
            text: "Delirium \u{2014} {2}{G}{G}: This land becomes a copy of target permanent card in your graveyard until end of turn. Activate only if there are four or more card types among cards in your graveyard.",
            cost: Cost::mana(mana(2, 0, 0, 0, 0, 2)),
            targets: &[TargetSpec { min: 1, max: 1, filter: permanent_card_in_your_graveyard }],
            effect: become_copy,
            condition: Some(has_delirium),
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::SHIFTING_WOODLAND
};
