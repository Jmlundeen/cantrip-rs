use cantrip_api::*;
use cantrip_mechanics::history::in_your_first_turns;
use cantrip_mechanics::{choose, copy, delayed, effects, plot};
use cantrip_oracle as oracle;

/// "You can't cast Jace Reawakened during your first, second, or third turns of the game" (CR 601.3, 500): a cast rule on the
/// caster's own turns only, so another player's turn (with some other effect letting it be cast) is not covered, and an extra
/// turn counts as a turn taken.
fn not_in_your_first_three_turns(q: &dyn Query, asker: &Asker) -> bool {
    !in_your_first_turns(q, asker.controller, 3)
}

/// "+1: You may exile a nonland card with mana value 3 or less from your hand. If you do, it becomes plotted." (CR 702.170a) It
/// is a choice made as the ability resolves (CR 608.2d): declining is the empty answer to an up-to-one choice, and nothing is
/// asked of a hand with no such card. The card is exiled and then plotted, castable free as a sorcery on a later turn.
fn exile_and_plot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&card| !g.chars(card).is(CardType::Land) && g.mana_value(card) <= 3).collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::Plot, "nonland", hand, eligible, 1, true)?;
    if let Some(&card) = chosen.first() {
        let exiled = g.exile(card);
        plot::become_plotted(g, exiled);
    }
    Ok(())
}

/// "\u{2212}6: Until end of turn, whenever you cast a spell, copy it." (CR 603.7b): sets up a delayed triggered ability for the
/// turn, controlled by the ability's controller.
fn copy_spells_this_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

/// "Whenever you cast a spell" (CR 601.2i, 603.2): the spell is the trigger's subject, so the copy is of that spell.
fn you_cast_a_spell(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == me.controller => Fired::about(Target::Object(*spell)),
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    cast_only_if: Some(not_in_your_first_three_turns),
    abilities: &[
        ability(loyalty(1), effects::loot, "+1: Draw a card, then discard a card."),
        ability(loyalty(1), exile_and_plot, "+1: You may exile a nonland card with mana value 3 or less from your hand. If you do, it becomes plotted."),
        ability(loyalty(-6), copy_spells_this_turn, "\u{2212}6: Until end of turn, whenever you cast a spell, copy it. You may choose new targets for the copy."),
    ],
    delayed: &[triggered(
        when_asked(EventMask::SPELL_CAST, you_cast_a_spell),
        copy::copy_the_cast_spell,
        "Whenever you cast a spell, copy it. You may choose new targets for the copy.",
    )],
    ..oracle::JACE_REAWAKENED
};
