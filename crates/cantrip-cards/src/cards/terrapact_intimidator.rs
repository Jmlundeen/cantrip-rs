use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens, triggers};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("HaveControllerCreateLanders", "have its controller create two Lander tokens?", "If you don't, it gets two +1/+1 counters", "Let them create two Landers", "Put two +1/+1 counters on it");

/// "Target opponent may have you create two Lander tokens. If they don't, put two +1/+1 counters on this creature" (CR 608.2d,
/// 701.7): the opponent decides as it resolves, and the two tokens come together (CR 111.1). The counters go on only if the Intimidator is
/// still the permanent that entered (CR 400.7); nothing happens if the target opponent is no longer legal (CR 608.2b).
fn offer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(them) = item.target(0) else { return Ok(()) };
    if effects::ask_may(g, them, item.object, Purpose::Ask(&ASK))? {
        tokens::create(g, item.controller, &tokens::LANDER, 2)?;
    } else if targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, offer, "When this creature enters, target opponent may have you create two Lander tokens. If they don't, put two +1/+1 counters on this creature.")
        .targets(&[targets::OPPONENT])],
    ..oracle::TERRAPACT_INTIMIDATOR
};
