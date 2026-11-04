use cantrip_api::*;
use cantrip_mechanics::triggers::this_or_another_of_subtype_you_control_enters;
use cantrip_mechanics::{lands, library, warp};
use cantrip_oracle as oracle;

/// "Exile the top card of your library. For as long as that card remains exiled, you may play it if you control a Kavu" (CR 406,
/// 611.2, 601.3, 305.1): the card is exiled face up, and while it stays in exile its controller may play it - a land as a land
/// play too - at the times the rules allow and paying its costs (ruling 2025-07-25), whenever they control a Kavu as they would
/// play it. The permission outlasts this creature (ruling 2025-07-25). "Kavu" is read through the source's words (CR 612).
fn exile_the_top_card_play_it_while_you_control_a_kavu(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let kavu = g.words(item.object).subtype(Subtype::Kavu);
    let grant = PlayGrant { lands: true, when: Some(PlayCondition::ControlsSubtype(kavu)), ..PlayGrant::CAST };
    library::exile_top_may_play(g, item.controller, 1, item.controller, grant);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, this_or_another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Kavu) }>),
        exile_the_top_card_play_it_while_you_control_a_kavu,
        "Whenever this creature or another Kavu you control enters, exile the top card of your library. For as long as that card remains exiled, you may play it if you control a Kavu.",
    )],
    ways: &[warp::way(mana("{1}{R}"), "Warp {1}{R}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::POSSIBILITY_TECHNICIAN
};
