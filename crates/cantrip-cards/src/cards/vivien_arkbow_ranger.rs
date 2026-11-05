use cantrip_api::*;
use cantrip_mechanics::{effects, fight, outside, select, targets};
use cantrip_oracle as oracle;

/// "+1: Distribute two +1/+1 counters among up to two target creatures. They gain trample until end of turn."
///
/// The targets and their shares were fixed as the ability was put on the stack (CR 601.2d, 602.2b); a target that has become
/// illegal since takes its share with it (CR 608.2b). The counters are one sentence and the trample another (CR 608.2c): the
/// creatures that gain it are those still legal targets.
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::distribute_plus_one_counters(g, item)?;
    g.end_sentence();
    instruction(g, |g| {
        for slot in 0..item.target_count as usize {
            if let Some(creature) = targets::object(g, item, slot) {
                effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Trample);
            }
        }
    });
    Ok(())
}

/// "-5: You may reveal a creature card you own from outside the game and put it into your hand." The creature card chosen is
/// revealed (CR 701.20a) and brought in (CR 400.11b).
fn minus_five(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    outside::may_reveal_card_into_hand(g, item.controller, item.object, Purpose::PutIntoHand, select::CREATURE_CARD)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), plus_one, "+1: Distribute two +1/+1 counters among up to two target creatures. They gain trample until end of turn.")
            .targets(&[targets::CREATURE.between(0, 2)])
            .divide(Divide { total: 2, reads: None }),
        ability(loyalty(-3), fight::first_target_bites_second, "\u{2212}3: Target creature you control deals damage equal to its power to target creature or planeswalker.")
            .targets(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_OR_PLANESWALKER]),
        ability(loyalty(-5), minus_five, "\u{2212}5: You may reveal a creature card you own from outside the game and put it into your hand."),
    ],
    ..oracle::VIVIEN_ARKBOW_RANGER
};
