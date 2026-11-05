use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// Pay {1}{G} for first strike, or the creatures blocking or blocked by this creature gain it (CR 611.2a). Which creatures those
/// are is read as the Slug last existed on the battlefield if it has left it (CR 608.2h), both the creatures it blocks and the
/// ones blocking it.
fn resolve(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let paid = g.may_pay(item.controller, item.object, item.object, cost("{1}{G}"), &PAY_QUESTION)?;
    g.end_sentence();
    if paid {
        effects::this_gains_first_strike(g, item)?;
    } else {
        let mut creatures = g.creatures_blocked_by(item.object);
        for id in g.creatures_blocking(item.object) {
            if !creatures.contains(&id) {
                creatures.push(id);
            }
        }
        creatures.retain(|&id| g.chars(id).is(CardType::Creature));
        instruction(g, |g| {
            for id in creatures {
                effects::gain_until_end_of_turn(g, item.object, id, Keyword::FirstStrike);
            }
        });
    }
    Ok(())
}

static PAY_QUESTION: Question = Question::new("Pay", "pay {cost}?", "", "Pay {cost}", "Decline");

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask(EventMask::BLOCKS.0 | EventMask::BECAME_BLOCKED.0), triggers::this_blocks_or_becomes_blocked),
        resolve,
        "Whenever this creature blocks or becomes blocked, you may pay {1}{G}. If you do, this creature gains first strike until end of turn. Otherwise, each creature blocking or blocked by this creature gains first strike until end of turn.",
    )],
    ..oracle::SPITTING_SLUG
};
