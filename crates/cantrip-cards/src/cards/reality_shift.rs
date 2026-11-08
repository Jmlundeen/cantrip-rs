use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Exile target creature. Its controller manifests the top card of their library." (CR 701.40a, 608.2h): the exiling is
/// what a legal target allows, and the manifest belongs to the exiled creature's controller - read as it last was, so an
/// opponent's creature manifests for that opponent. One card, so no looking at two: the top card becomes a face-down 2/2
/// creature (CR 708.2) and nothing when the library is empty.
fn exile_and_manifest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(creature);
    g.exile(creature);
    g.end_sentence();
    let Some(&top) = g.state().zone(Zone::library(controller)).first() else { return Ok(()) };
    g.put_onto_battlefield_face_down(top, controller, FaceDown::Manifested, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], exile_and_manifest),
    ..oracle::REALITY_SHIFT
};
