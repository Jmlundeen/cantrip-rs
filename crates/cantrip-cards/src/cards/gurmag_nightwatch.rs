use cantrip_api::*;
use cantrip_mechanics::top_of_library;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Look at the top three cards of your library. You may put one of those cards back on top of your library. Put the rest
/// into your graveyard." A library of fewer than three shows what it has (CR 609.3) and one of none does nothing. The card
/// kept is left where it is, the top of what remains once the rest have gone; the rest go to the graveyard together, in the
/// order their owner arranges them (CR 404.3).
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let looked = top_of_library(g, player, 3);
    if looked.is_empty() {
        return Ok(());
    }
    g.look_at(player, &looked);
    let ask = RequestKind::DistributeCards {
        source: item.object,
        cards: looked.clone(),
        destinations: vec![
            Destination { kind: DestinationKind::LibraryTop, min: 0, max: 1, eligible: looked.clone() },
            Destination { kind: DestinationKind::Graveyard, min: 0, max: u8::MAX, eligible: looked.clone() },
        ],
        may_shuffle: false,
    };
    let mut kept = Vec::new();
    if let Answer::Placements { placements, .. } = g.ask(player, ask)? {
        kept = placements[0].clone();
    }
    let rest: Vec<ObjectId> = looked.into_iter().filter(|c| !kept.contains(c)).collect();
    g.put_into_graveyard(&rest);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, look, "When this creature enters, look at the top three cards of your library. You may put one of those cards back on top of your library. Put the rest into your graveyard.")],
    ..oracle::GURMAG_NIGHTWATCH
};
