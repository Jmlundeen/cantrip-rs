use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, library};
use cantrip_oracle as oracle;

/// "Target exiled card with warp" (CR 115.1, 702.185a): a card in exile, any player's, that has warp, whether or not warp
/// exiled it (the ruling). A card exiled face down has no characteristics, so no warp (CR 406.3a).
fn exiled_card_with_warp(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if {
        q.obj(id).zone.is(ZoneKind::Exile) && q.card(q.chars(id).card).ways.iter().any(|w| w.kind == WayKind::Warp)
    })
}

static MODAL: ModalDef = modes(&[
    mode(effects::two_plus_one_counters_on_this, "Put two +1/+1 counters on this creature."),
    mode(library::put_first_target_on_the_bottom, "Put target exiled card with warp on the bottom of its owner's library.")
        .targets(&[target(exiled_card_with_warp)]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When this creature enters, choose one \u{2014} Put two +1/+1 counters on this creature; or put target exiled card with warp on the bottom of its owner's library.",
    )],
    ..oracle::BLADE_OF_THE_SWARM
};
