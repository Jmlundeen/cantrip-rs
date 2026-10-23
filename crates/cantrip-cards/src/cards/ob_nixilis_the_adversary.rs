use cantrip_api::*;
use cantrip_mechanics::{casualty, choose_from_hand, damage_target, effects, players, targets, tokens};
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

const DISCARD: Question = Question::new("DiscardInsteadOfLosingLife", "discard a card instead of losing 2 life?", "Otherwise you lose 2 life", "Discard a card", "Lose 2 life");

/// The 1/1 red Devil creature token with "When this token dies, it deals 1 damage to any target."
pub static DEVIL: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[triggered(THIS_DIES, devil_dies, "When this token dies, it deals 1 damage to any target.").targets(&[targets::ANY])],
    text: "When this token dies, it deals 1 damage to any target.",
    ..token!(Devil)
};

/// "It deals 1 damage to any target": dealt by the token as it last existed (CR 603.10a, 608.2h).
fn devil_dies(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

/// "The copy isn't legendary and has starting loyalty X" (CR 707.9b, 707.10f): the copy of the spell, a token as it resolves,
/// with X as announced for this spell. X is announced no higher than a power can be (`i16::MAX`), so a loyalty holds it.
fn copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    // The announcement is capped at `i16::MAX` (`VariableCost::SacrificeWithPowerX`), so `try_from` cannot fail.
    let Ok(loyalty) = i16::try_from(casualty::x(g, item)) else { return Ok(()) };
    let values = g.copiable_values(item.object).nonlegendary().with_loyalty(loyalty);
    g.copy_spell_as(item.object, item.controller, true, values)?;
    Ok(())
}

/// +1: "Each opponent loses 2 life unless they discard a card. If you control a Demon or Devil, you gain 2 life."
/// Each opponent in turn order (CR 101.4) chooses whether to discard (nothing is asked of an empty hand, which cannot),
/// then the discards happen together and the life is lost (CR 608.2d). The Demon or Devil is by the words of this card (CR 612).
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let opponents: Vec<PlayerId> = g.state().apnap().filter(|&p| g.state().opponents(you).any(|o| o == p)).collect();
    let (mut discards, mut losers) = (Vec::new(), Vec::new());
    for opponent in opponents {
        if !players::reached(g, opponent) {
            continue;
        }
        if g.state().zone(Zone::hand(opponent)).is_empty() || !effects::ask_may(g, opponent, item.object, Purpose::Ask(&DISCARD))? {
            losers.push(opponent);
        } else {
            discards.extend(choose_from_hand(g, opponent, item.object, 1)?);
        }
    }
    g.discard_together(&discards);
    for player in losers {
        g.lose_life(player, 2);
    }
    g.end_sentence();
    let words = g.words(item.object);
    let (demon, devil) = (words.subtype(Subtype::Demon), words.subtype(Subtype::Devil));
    if g.permanents_of(you).into_iter().any(|id| g.chars(id).has_subtype(demon) || g.chars(id).has_subtype(devil)) {
        g.gain_life(you, 2);
    }
    Ok(())
}

/// \-2: "Create a 1/1 red Devil creature token with ..."
fn minus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &DEVIL, 1)?;
    Ok(())
}

/// \-7: "Target player draws seven cards and loses 7 life."
fn minus_seven(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        g.draw(player, 7);
        g.lose_life(player, 7);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    optional_cost: Some(casualty::X),
    labels: &["Cast without casualty", "Casualty X: sacrifice a creature with power X or greater"],
    abilities: &[
        casualty::trigger_with("Casualty X. The copy isn't legendary and has starting loyalty X.", copy),
        ability(loyalty(1), plus_one, "+1: Each opponent loses 2 life unless they discard a card. If you control a Demon or Devil, you gain 2 life."),
        ability(loyalty(-2), minus_two, "−2: Create a 1/1 red Devil creature token with \"When this token dies, it deals 1 damage to any target.\""),
        ability(loyalty(-7), minus_seven, "−7: Target player draws seven cards and loses 7 life.").targets(&[targets::PLAYER]),
    ],
    ..oracle::OB_NIXILIS_THE_ADVERSARY
};

pub static EXTRA: &[&CardDef] = &[&DEVIL];
