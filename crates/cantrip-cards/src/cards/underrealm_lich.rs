use cantrip_api::*;
use cantrip_mechanics::effects::this_gains_indestructible_then_tap;
use cantrip_mechanics::replacements::{instead_of_drawing, you_would_draw_a_card};
use cantrip_mechanics::top_of_library;
use cantrip_oracle as oracle;

/// "... instead look at the top three cards of your library, then put one into your hand and the rest into your
/// graveyard" (CR 614.1a, 401.2): no card is drawn (CR 121.5), so nothing that triggers on drawing does. A library of fewer
/// than three shows what it has (CR 609.3), and an empty one shows nothing: the draw is replaced all the same, so its
/// player does not lose for it (its ruling).
fn look_instead(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    let _ = look_take_one_rest_to_graveyard(g, asker.controller, asker.source);
}

fn look_take_one_rest_to_graveyard(g: &mut dyn Game, player: PlayerId, source: ObjectId) -> Result<(), Suspend> {
    let looked = top_of_library(g, player, 3);
    if looked.is_empty() {
        return Ok(());
    }
    let ask = RequestKind::DistributeCards {
        source,
        cards: looked.clone(),
        destinations: vec![
            Destination { kind: DestinationKind::Hand, min: 1, max: 1, eligible: looked.clone() },
            Destination { kind: DestinationKind::Graveyard, min: 0, max: u8::MAX, eligible: looked.clone() },
        ],
        may_shuffle: false,
    };
    let mut taken = Vec::new();
    if let Answer::Placements { placements, .. } = g.ask(player, ask)? {
        taken = placements[0].clone();
    }
    // "Put one into your hand and the rest into your graveyard" is two actions, followed in the order written (CR 608.2c):
    // the card into the hand, then the rest into the graveyard together (CR 608.2f), in the order their owner arranges as
    // they arrive (CR 404.3, asked by the move itself). Looked at, not revealed.
    g.return_to_hand(&taken);
    let rest: Vec<(ObjectId, Zone)> =
        looked.iter().copied().filter(|c| !taken.contains(c)).map(|c| (c, Zone::graveyard(g.obj(c).owner))).collect();
    g.move_together(&rest, MoveReason::Other);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(life(4), this_gains_indestructible_then_tap, "Pay 4 life: This creature gains indestructible until end of turn. Tap it.")],
    // "If you would draw a card, instead look at the top three cards of your library, then put one into your hand and the
    // rest into your graveyard." (CR 121.6, 614.1a)
    replacements: &[replacement(
        ReplacementKind::Draw,
        you_would_draw_a_card,
        instead_of_drawing,
        "puts one of the top three cards into hand instead, and the rest into the graveyard",
    )
    .then(look_instead)],
    ..oracle::UNDERREALM_LICH
};
