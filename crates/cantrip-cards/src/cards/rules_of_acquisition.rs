use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Gain control of target nonland permanent an opponent controls. That opponent draws three cards" (CR 613.1b, 611.2a,
/// 121.1): with no duration the control change lasts; the cards are drawn by the permanent's controller as the spell
/// resolves, read before control changes.
fn follow_the_third_rule(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let opponent = g.controller(permanent);
    g.gain_control(item.object, permanent, item.controller, Duration::Indefinite);
    g.draw(opponent, 3);
    Ok(())
}

/// "Target opponent gains control of target nonland permanent you control. If they do, you draw three cards"
/// (CR 613.1b, 608.2b): the opponent cannot refuse, so they do unless a target is gone - then nothing is gained and no
/// card is drawn.
fn follow_the_98th_rule(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (Some(opponent), Some(permanent)) = (targets::player(g, item, 0), targets::object(g, item, 1)) else { return Ok(()) };
    g.gain_control(item.object, permanent, opponent, Duration::Indefinite);
    g.draw(item.controller, 3);
    Ok(())
}

/// "Choose one" (CR 700.2): each mode has its own targets (CR 700.2c).
static MODAL: ModalDef = modes(&[
    mode(
        follow_the_third_rule,
        "Follow the 3rd Rule \u{2014} Gain control of target nonland permanent an opponent controls. That opponent draws three cards.",
    )
    .targets(&[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS]),
    mode(
        follow_the_98th_rule,
        "Follow the 98th Rule \u{2014} Target opponent gains control of target nonland permanent you control. If they do, you draw three cards.",
    )
    .targets(&[
        targets::OPPONENT.labelled("Opponent who gains control of the permanent"),
        target(targets::nonland_permanent_you_control_filter).labelled("Nonland permanent you control for that opponent to gain control of"),
    ]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::RULES_OF_ACQUISITION };
