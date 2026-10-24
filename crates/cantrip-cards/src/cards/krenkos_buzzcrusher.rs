use cantrip_api::*;
use cantrip_mechanics::lands::each_may_search_basic_lands_together;
use cantrip_mechanics::select::{self, Part, Selection};
use cantrip_mechanics::{choose, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

fn is_nonbasic_land(q: &dyn Query, id: ObjectId) -> bool {
    let chars = q.chars(id);
    chars.is(CardType::Land) && !chars.supertypes.basic
}

fn nonbasic_land(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |land| is_nonbasic_land(q, land))
}

/// "A nonbasic land": a land without the basic supertype (CR 205.4a).
const NONBASIC_LAND: Part = select::part(&(nonbasic_land as SelectFn), 0, "");

/// "When this creature enters, for each player, destroy up to one nonbasic land that player controls. For each land destroyed
/// this way, its controller may search their library for a basic land card, put it onto the battlefield tapped, then shuffle."
/// Not targeted (the ruling): you choose, for each player in turn order, up to one of the nonbasic lands they control as the
/// ability resolves (CR 608.2d), and the lands chosen are destroyed together (CR 701.8, 101.4). Each land's controller, read
/// before it is destroyed (CR 608.2h), then may search for a basic land; only a land that was destroyed counts (the new object its destruction made, CR 400.7) (an indestructible
/// one stays and its controller does not search). The players act in turn order, each deciding and searching before the next (CR 101.4c),
/// the lands found enter at once and each library that was searched is shuffled after (CR 701.23).
fn destroy_nonbasic_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut chosen: Vec<ObjectId> = Vec::new();
    let players: Vec<PlayerId> = g.state().apnap().collect();
    for player in players {
        let lands: Vec<ObjectId> = g.permanents_of(player).into_iter().filter(|&p| is_nonbasic_land(&*g, p)).collect();
        let rule = Selection::of(select::up_to(1), &[NONBASIC_LAND]).called("nonbasic land");
        chosen.extend(choose::select(g, item.controller, item.object, Purpose::Destroy, lands, rule)?);
    }
    let controllers: Vec<(ObjectId, PlayerId)> = chosen.iter().map(|&land| (land, g.controller(land))).collect();
    let destroyed = g.destroy_all(&chosen);
    g.end_sentence();
    let searchers: Vec<(PlayerId, u32)> = controllers
        .iter()
        .filter(|&&(land, _)| g.obj(land).became.is_some_and(|new| destroyed.contains(&new)))
        .map(|&(_, player)| (player, 1))
        .collect();
    each_may_search_basic_lands_together(g, &searchers, item.object, true)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        destroy_nonbasic_lands,
        "When this creature enters, for each player, destroy up to one nonbasic land that player controls. For each land destroyed this way, its controller may search their library for a basic land card, put it onto the battlefield tapped, then shuffle.",
    )],
    ..oracle::KRENKOS_BUZZCRUSHER
};
