use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "As this creature enters, choose a player" (CR 614.12a): its controller chooses, among every player, before anyone
/// receives priority. A game with no one else to choose is not asked.
fn choose_a_player(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let candidates: Vec<PlayerId> = g.state().apnap().collect();
    entering.chosen_player = match candidates.len() {
        1 => Some(candidates[0]),
        _ => choose::player(g, entering.controller, entering.object, candidates, "Protection", 0)?,
    };
    Ok(())
}

fn protection_from_the_chosen_player(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    if let Some(player) = q.state().obj(source).marks.chosen_player {
        chars.protection_players |= 1 << player.0;
    }
}

pub static CARD: CardDef = CardDef {
    enters: Some(choose_a_player),
    // "This creature has protection from the chosen player."
    statics: &[characteristic_defining(Layer::Ability, protection_from_the_chosen_player, "This creature has protection from the chosen player.")],
    ..oracle::TRUE_NAME_NEMESIS
};
