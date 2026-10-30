use cantrip_api::*;
use cantrip_mechanics::{pump, triggers};
use cantrip_oracle as oracle;

/// "If a creature you control would enter while you control an attacking creature, instead it enters tapped and
/// attacking" (CR 614.1d, 508.4): the condition is judged as the permanent would arrive, on the board as it is, and the
/// replacement is Kira's own, so it functions only while she is on the battlefield (CR 604.1) - she does not turn her own
/// arrival tapped. The tapped part is what `ZoneChange::tapped` carries. The engine has no route for a replacement to put
/// an arriving permanent onto the battlefield attacking: an attacker needs a defender chosen as it enters
/// (`combat::choose_attack_target`), which only `Game::enter_attacking`, called after the permanent has arrived, takes.
/// Scenarios/kira-nerys-firebrand-enters-tapped-and-attacking.cantrip holds the rules-correct outcome as a known bug.
fn a_creature_of_yours_enters_while_attacking(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.to.is(ZoneKind::Battlefield)
            && z.controller == asker.controller
            && q.chars(z.object).is(CardType::Creature))
        && q.state().battlefield().iter().any(|&id| q.controller(id) == asker.controller && q.is_attacking(id))
}

/// "…instead it enters tapped" (CR 614.1c): the one part of the replacement the arrival carries.
fn enters_tapped(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.tapped = true;
    }
}

/// "Whenever another creature you control enters, it gets +1/+1 until end of turn." (CR 603.6a, 611.2): the entering
/// creature is the trigger's subject, and it is a new object if it has left and returned since (CR 400.7).
fn pump_the_entering_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(entering) = item.subject {
        pump(g, item.object, entering, 1, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS,
        pump_the_entering_creature,
        "Whenever another creature you control enters, it gets +1/+1 until end of turn.",
    )],
    replacements: &[replacement(
        ReplacementKind::ZoneChange,
        a_creature_of_yours_enters_while_attacking,
        enters_tapped,
        "it enters tapped while you control an attacking creature",
    )],
    ..oracle::KIRA_NERYS_FIREBRAND
};
