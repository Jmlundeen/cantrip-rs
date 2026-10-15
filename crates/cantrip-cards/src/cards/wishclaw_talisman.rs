use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_oracle as oracle;

/// "This artifact enters with three wish counters on it" (CR 614.1c), however it enters.
fn enters_with_three_wish_counters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Wish as usize] += 3;
    Ok(())
}

/// "Activate only during your turn" (CR 602.5b).
fn during_your_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.state().turn.active == asker.controller
}

/// "Search your library for a card, put it into your hand, then shuffle. An opponent gains control of this artifact."
/// The opponent is chosen from those still in the game; one is no choice. Nothing changes hands if the artifact has
/// left the battlefield (CR 608.2b).
fn search_and_give(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    search_to_hand(g, player, item.object, Find::ANY, |_, _| true)?;

    let me = item.object;
    if !is_live(g, me) || !g.is_on_battlefield(me) {
        return Ok(());
    }
    let opponents: Vec<PlayerId> = g.state().opponents(player).collect();
    let new_controller = match opponents.len() {
        1 => Some(opponents[0]),
        _ => choose::player(g, player, me, opponents, "gains control of this artifact.", 0)?,
    };
    let Some(new_controller) = new_controller else { return Ok(()) };
    g.gain_control(me, me, new_controller, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_three_wish_counters),
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{1}, {T}, Remove a wish counter from this artifact: Search your library for a card, put it into your hand, then shuffle. An opponent gains control of this artifact. Activate only during your turn.",
        cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, remove_counters: Some((CounterKind::Wish, 1)), ..Cost::NONE },
        condition: Some(during_your_turn),
        effect: search_and_give,
        ..ActivatedDef::BLANK
    })],
    ..oracle::WISHCLAW_TALISMAN
};
