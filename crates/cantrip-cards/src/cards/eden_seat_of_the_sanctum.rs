use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Sacrifice", "sacrifice this land?", "When you do, return another target permanent card from your graveyard to your hand", "Sacrifice", "Don't sacrifice");

/// "Mill two cards. Then you may sacrifice this land. When you do, return another target permanent card from your graveyard to your
/// hand" (CR 701.17a, 701.21a, 603.12): the reflexive trigger exists only if the land was really sacrificed, and chooses its target
/// once the milled cards and this land are in the graveyard.
fn mill_then_may_sacrifice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 2);
    if effects::ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? && !g.sacrifice(item.controller, &[item.object]).is_empty() {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

/// "Another target permanent card in your graveyard": the land just sacrificed, which the ability's source became (CR 400.7), is
/// not another card.
fn another_permanent_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_card_in_your_graveyard(q, asker, t) && Some(t) != q.obj(asker.source).became.map(Target::Object)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{5}").and(TAP),
            mill_then_may_sacrifice,
            "{5}, {T}: Mill two cards. Then you may sacrifice this land. When you do, return another target permanent card from your graveyard to your hand.",
        ),
    ],
    delayed: &[delayed_trigger(
        effects::return_first_target_to_hand,
        "When you do, return another target permanent card from your graveyard to your hand.",
    )
    .targets(&[target(another_permanent_card_in_your_graveyard)])],
    ..oracle::EDEN_SEAT_OF_THE_SANCTUM
};
