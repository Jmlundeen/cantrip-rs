use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "When Tuvok enters during the declare attackers step" (CR 603.2): the step is part of the trigger event, so Tuvok entering in
/// any other step triggers nothing. It is also checked again as the ability resolves, which changes nothing: a step ends
/// only with the stack empty (CR 500.2), so an ability that triggered in the declare attackers step resolves in it.
fn during_declare_attackers(q: &dyn Query, _: &Asker) -> bool {
    q.state().turn.step() == Step::DeclareAttackers
}

/// "Choose up to two target attacking creatures. For each of them, you may reselect which player or permanent that creature is
/// attacking." (CR 508.4, 508.5, 608.2b): each target is judged on its own as this resolves, and each is asked about on its
/// own, one at a time; an answer of none leaves that creature attacking what it was, and a target that has left combat since
/// is passed over. The players and permanents offered are the ones that creature's controller could attack, so no creature is
/// ever made to attack its controller.
fn reselect_what_it_attacks(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for slot in 0..2 {
        // A target that has gone, or is no longer attacking, is passed over (CR 608.2b).
        let Some(creature) = targets::object(g, item, slot).filter(|&c| g.is_attacking(c)) else { continue };
        let candidates = g.enter_attacking_targets(g.obj(creature).controller);
        let ask = RequestKind::ChooseTarget {
            replacing: None,
            label: "Reselect which player or permanent that creature is attacking (choose none to leave it attacking what it was).",
            slot_player: None,
            together: None,
            source: item.object,
            purpose: Purpose::ReselectAttackTarget,
            candidates,
            optional: true,
            least: 0,
            count: 1,
            any_number: false,
        };
        let Answer::Targets(chosen) = g.ask(item.controller, ask)? else { continue };
        if let Some(&defender) = chosen.first() {
            g.set_attack_target(creature, defender);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        reselect_what_it_attacks,
        "When Tuvok enters during the declare attackers step, choose up to two target attacking creatures. For each of them, you may reselect which player or permanent that creature is attacking. (A creature can't attack its controller.)",
    )
    .targets(&[targets::ATTACKING_CREATURE.between(0, 2)])
    .condition(during_declare_attackers)],
    ..oracle::TUVOK_CHIEF_TACTICAL_OFFICER
};
