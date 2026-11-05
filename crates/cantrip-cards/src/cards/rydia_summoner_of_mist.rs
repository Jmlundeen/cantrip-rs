use cantrip_api::*;
use cantrip_mechanics::{effects, haste_until_end_of_turn, targets, triggers};
use cantrip_oracle as oracle;

/// "Target Saga card with mana value X from your graveyard" (CR 107.3a, 202.3, 115.1): the slot of an ability with {X} in its
/// cost, `.targets_after_x()`, judged again on resolution with its own announced X.
fn saga_card_with_mana_value_x_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[])
        && matches!(t, Target::Object(id) if q.chars(id).has_subtype(Subtype::Saga) && targets::mana_value_is_x(q, id))
}

/// "Return target Saga card with mana value X from your graveyard to the battlefield with a finality counter on it. It gains
/// haste until end of turn": the counter is on it as it enters (CR 122.6, 614.1c). Nothing if the target is gone (CR 608.2b).
fn summon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let returned = g.put_onto_battlefield_with_counters(card, item.controller, false, &[(CounterKind::Finality, 1)])?;
    haste_until_end_of_turn(g, item.object, returned);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::LANDFALL,
            |g, item| effects::may_discard_to_draw(g, item, 1),
            "Landfall \u{2014} Whenever a land you control enters, you may discard a card. If you do, draw a card.",
        ),
        ability(
            cost("{X}").and(TAP),
            summon,
            "Summon \u{2014} {X}, {T}: Return target Saga card with mana value X from your graveyard to the battlefield with a finality counter on it. It gains haste until end of turn. Activate only as a sorcery.",
        )
        .targets(&[target(saga_card_with_mana_value_x_in_your_graveyard)])
        .targets_after_x()
        .sorcery_speed(),
    ],
    ..oracle::RYDIA_SUMMONER_OF_MIST
};
