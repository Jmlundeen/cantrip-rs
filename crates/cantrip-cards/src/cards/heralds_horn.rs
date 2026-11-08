use cantrip_api::*;
use cantrip_mechanics::chosen_type::enters_choosing_a_creature_type;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::top_of_library;
use cantrip_mechanics::triggers::YOUR_UPKEEP;
use cantrip_oracle as oracle;

const REVEAL: Question = Question::new(
    "RevealChosenTypeToHand",
    "reveal it and put it into your hand?",
    "It is a creature card of the chosen type. If you don't, it stays on top of your library",
    "Reveal it",
    "Don't",
);

/// "As this artifact enters, choose a creature type." (CR 614.12a): any creature type, chosen by the artifact's controller
/// as it enters and kept on it (CR 607.2a).
///
/// "Creature spells you cast of the chosen type cost {1} less to cast." (CR 601.2f, 205.3, 118.7a): only its controller's
/// creature spells (CR 308.1: a Kindred spell is no creature spell), judged on the spell's own subtypes as it is cast, and
/// the reduction is generic mana only, so a cost with none is not reduced (the 2017-08-25 ruling).
fn creature_spells_of_the_chosen_type_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) || !q.chars(spell).is(CardType::Creature) {
        return CostDelta::NONE;
    }
    match q.state().chosen_subtype(q.text_linked(source)) {
        Some(chosen) if q.chars(spell).has_subtype(chosen) => CostDelta::less(1),
        _ => CostDelta::NONE,
    }
}

/// "At the beginning of your upkeep, look at the top card of your library. If it's a creature card of the chosen type, you
/// may reveal it and put it into your hand." (CR 701.20a, 121.1): the card is looked at whatever it is, may be left
/// unrevealed, and is put back on top of the library without being revealed if you don't (the 2017-08-25 ruling). An empty
/// library has nothing to look at.
fn look_at_the_top_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if let Some(&top) = top_of_library(g, you, 1).first() {
        g.look_at(you, &[top]);
        let asker = g.item_asker(item, you);
        let of_the_chosen_type = g.chars(top).is(CardType::Creature)
            && g.state().chosen_subtype(g.linked_to(&asker)).is_some_and(|chosen| g.chars(top).has_subtype(chosen));
        if of_the_chosen_type && ask_may(g, you, item.object, Purpose::Ask(&REVEAL))? {
            g.reveal(you, &[top]);
            g.return_to_hand_revealed(&[top]);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_choosing_a_creature_type),
    cost_changes: &[creature_spells_of_the_chosen_type_cost_less],
    abilities: &[triggered(
        YOUR_UPKEEP,
        look_at_the_top_card,
        "At the beginning of your upkeep, look at the top card of your library. If it's a creature card of the chosen type, you may reveal it and put it into your hand.",
    )],
    ..oracle::HERALDS_HORN
};
