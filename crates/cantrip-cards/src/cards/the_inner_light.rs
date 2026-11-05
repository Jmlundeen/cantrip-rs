use cantrip_api::*;
use cantrip_mechanics::{effects, greatest_power, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "Target green or white creature card from your graveyard" (CR 115.1, 105.2): a creature card in your graveyard that is
/// either color, or both, the color words as this Saga's text says them (CR 612).
fn green_or_white_creature_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    let words = q.words(asker.source);
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature])
        && (q.chars(id).colors.has(words.color(Color::Green)) || q.chars(id).colors.has(words.color(Color::White)))
}

/// Chapter I: "Return target green or white creature card from your graveyard to the battlefield." (CR 608.2b): a target gone
/// by resolution is left alone.
fn i_return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_first_target_to_battlefield(g, item)
}

/// Chapter II: "Choose target creature. Whenever it deals combat damage to a player this turn, you draw that many cards."
/// (CR 603.7b): a delayed triggered ability made as the chapter resolves, watching that creature for the rest of the turn and
/// triggering each time it deals combat damage — its target is chosen now, and it is that creature, not the Saga, that the
/// ability is asked of (`DelayedWhen::WatchingThisTurn`).
fn ii_watch_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let this = g.card_id(&CARD);
    g.delayed_trigger(
        DelayedWhen::WatchingThisTurn,
        item.object,
        item.controller,
        AbilityRef::delayed(this, 0),
        Target::Object(creature),
    );
    Ok(())
}

/// "You draw that many cards" (CR 121.1): as many cards as the combat damage was, the trigger's `data[0]`.
fn draw_that_many(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, item.data[0]);
    Ok(())
}

/// Chapter III: "You gain life equal to the greatest power among creatures you control. Create a Treasure token." (CR 119.3,
/// 608.2m): the power is read as the chapter resolves, and the life is one gain.
fn iii_life_and_treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let greatest = greatest_power::among_creatures_you_control(g, item.controller);
    g.gain_life(item.controller, greatest as i32);
    tokens::create_treasure(g, item.controller)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, i_return_it, "I \u{2014} Return target green or white creature card from your graveyard to the battlefield.")
            .targets(&[target(green_or_white_creature_card_in_your_graveyard)]),
        chapter(
            2,
            ii_watch_it,
            "II \u{2014} Choose target creature. Whenever it deals combat damage to a player this turn, you draw that many cards.",
        )
        .targets(&[targets::CREATURE]),
        chapter(
            3,
            iii_life_and_treasure,
            "III \u{2014} You gain life equal to the greatest power among creatures you control. Create a Treasure token.",
        ),
    ],
    delayed: &[triggered(
        triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT,
        draw_that_many,
        "Whenever it deals combat damage to a player this turn, you draw that many cards.",
    )],
    ..oracle::THE_INNER_LIGHT
};
