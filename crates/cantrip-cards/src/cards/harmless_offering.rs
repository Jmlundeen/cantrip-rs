use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target permanent you control" (CR 115.1, 110.2): a permanent on the battlefield that its caster controls, judged again as the
/// spell resolves (CR 608.2b), so one whose control changed in response is no longer a legal target.
fn permanent_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.controller(id) == asker.controller)
}

/// "Target opponent gains control of target permanent you control" (CR 108.4, 611.2: with no duration): if either target is
/// no longer legal there is nobody to take it, or nothing to take, and the other target is unaffected (CR 608.2b). The opponent
/// cannot refuse.
fn donate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (Some(opponent), Some(permanent)) = (targets::player(g, item, 0), targets::object(g, item, 1)) else { return Ok(()) };
    g.gain_control(item.object, permanent, opponent, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[
            targets::OPPONENT.labelled("Opponent who gains control of the permanent"),
            target(permanent_you_control).labelled("Permanent you control for that opponent to gain control of"),
        ],
        donate,
    ),
    ..oracle::HARMLESS_OFFERING
};
