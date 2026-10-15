use cantrip_api::*;
use cantrip_mechanics::library::reveal_top_take_one_rest_to_graveyard;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// The 0/1 colorless Eldrazi Spawn creature token with "Sacrifice this token: Add {C}."
pub static ELDRAZI_SPAWN: CardDef = CardDef {
    name: "Eldrazi Spawn",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Eldrazi, Subtype::Spawn]),
    power: Some(0),
    toughness: Some(1),
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice this token: Add {C}.",
        cost: Cost { sacrifice_self: true, ..Cost::NONE },
        mana: Some(mana::add_c),
        adds: ManaPool([0, 0, 0, 0, 0, 1]),
        ..ActivatedDef::BLANK
    })],
    text: "Sacrifice this token: Add {C}.",
    ..CardDef::BLANK
};

/// A permanent card (CR 110.1): one of a type that can exist on the battlefield.
fn permanent_card(chars: &Chars) -> bool {
    [CardType::Artifact, CardType::Battle, CardType::Creature, CardType::Enchantment, CardType::Land, CardType::Planeswalker]
        .into_iter()
        .any(|t| chars.is(t))
}

/// Reveal the top four cards of your library; you may put a permanent card from among them into your hand; put the rest
/// into your graveyard, in the order they were revealed. With no permanent card among them there is nothing to decide.
/// Then create an Eldrazi Spawn.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    reveal_top_take_one_rest_to_graveyard(g, player, item.object, 4, permanent_card)?;
    cantrip_mechanics::tokens::create(g, player, &ELDRAZI_SPAWN, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::MALEVOLENT_RUMBLE };

pub static EXTRA: &[&CardDef] = &[&ELDRAZI_SPAWN];
