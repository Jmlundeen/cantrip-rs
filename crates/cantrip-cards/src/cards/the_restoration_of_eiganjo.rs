use cantrip_api::*;
use cantrip_mechanics::{effects, exile_and_return_transformed, lands, library, select, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever this creature attacks or blocks, create a 1/1 colorless Spirit creature token." (CR 111.1).
fn create_1_1_colorless_spirit(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::SPIRIT_1_1_COLORLESS, 1)?;
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ATTACKS_OR_BLOCKS,
            create_1_1_colorless_spirit,
            "Whenever this creature attacks or blocks, create a 1/1 colorless Spirit creature token.",
        ),
    ],
    ..oracle::ARCHITECT_OF_RESTORATION
};

/// "I — Search your library for a basic Plains card, reveal it, put it into your hand, then shuffle." (CR 608.2).
fn i_search_library_basic_plains(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    library::find_to_hand(
        g,
        item.controller,
        item.object,
        select::BASIC_PLAINS_CARD,
        true,
    )?;
    Ok(())
}

/// "When you do, return target permanent card with mana value 2 or less from your graveyard to the battlefield tapped" (CR 115.1,
/// 603.12): the target is a permanent card of your graveyard with mana value 2 or less.
fn reflexive_2_target_0(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &targets::PERMANENT_CARD_TYPES, 2)
}

/// "When you do, return target permanent card with mana value 2 or less from your graveyard to the battlefield tapped" (CR 110.5b).
fn reflexive_2(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::return_target_land_card_from_graveyard_tapped(g, item, 0)
}

/// "II — You may discard a card. When you do, return target permanent card with mana value 2 or less from your graveyard to the
/// battlefield tapped." (CR 701.9a, 603.12): the reflexive trigger follows a card really discarded.
fn ii_discard_card_when_do(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::may_discard_a_card(g, item)? {
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(item.ability.card, 0), Target::None);
    }
    Ok(())
}

/// "III — Exile this Saga, then return it to the battlefield transformed under your control." (CR 712.14a).
fn iii_exile_saga_return_battlefield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_and_return_transformed(g, item.object, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(
            1,
            i_search_library_basic_plains,
            "I — Search your library for a basic Plains card, reveal it, put it into your hand, then shuffle.",
        ),
        chapter(
            2,
            ii_discard_card_when_do,
            "II — You may discard a card. When you do, return target permanent card with mana value 2 or less from your graveyard to the battlefield tapped.",
        ),
        chapter(
            3,
            iii_exile_saga_return_battlefield,
            "III — Exile this Saga, then return it to the battlefield transformed under your control.",
        ),
    ],
    delayed: &[
        delayed_trigger(
                reflexive_2,
                "When you do, return target permanent card with mana value 2 or less from your graveyard to the battlefield tapped.",
            )
            .targets(&[target(reflexive_2_target_0)]),
    ],
    back: Some(&BACK),
    ..oracle::THE_RESTORATION_OF_EIGANJO
};

pub static EXTRA: &[&CardDef] = &[&tokens::SPIRIT_1_1_COLORLESS];
