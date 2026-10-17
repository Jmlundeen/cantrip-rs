use cantrip_api::*;
use cantrip_mechanics::affinity;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "If you have fewer than four cards in hand" (CR 603.4): an intervening "if", checked as the ability triggers and again as it
/// resolves.
fn fewer_than_four_cards_in_hand(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::hand(asker.controller)).len() < 4
}

/// "Draw cards equal to the difference": four less the cards in hand now, as it resolves (CR 608.2c).
fn draw_the_difference(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let in_hand = g.state().zone(Zone::hand(item.controller)).len();
    g.draw(item.controller, 4u32.saturating_sub(in_hand as u32));
    Ok(())
}

/// "Krang gets +1/+0 for each other artifact you control" (CR 613.4c, layer 7c): the artifacts as the earlier layers made them,
/// under the control of Krang's controller, not Krang itself.
fn plus_power_per_other_artifact(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let you = state.obj(source).controller;
    let n = state.battlefield().iter().filter(|&&id| id != source && state.obj(id).controller == you && view.chars(id).is(CardType::Artifact)).count() as i16;
    chars.power = chars.power.map(|v| v + n);
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[affinity::for_artifacts],
    abilities: &[triggered(THIS_ENTERS, draw_the_difference, "When Krang enters, if you have fewer than four cards in hand, draw cards equal to the difference.")
        .condition(fewer_than_four_cards_in_hand)],
    statics: &[static_ability(Layer::PtModify, cantrip_mechanics::statics::itself, plus_power_per_other_artifact, "Krang gets +1/+0 for each other artifact you control.")],
    ..oracle::KRANG_MASTER_MIND
};
