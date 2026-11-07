use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

const TEXT: &str = "Artifact creatures you control have menace and \"Whenever a creature blocks this creature, that creature gets -1/-1 until end of turn.\"";

/// "Whenever a creature blocks this creature" (CR 509.3d, 603.2): once for each creature blocking it, that creature the subject.
fn blocks_this(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Blocks { blocker, attacker, .. } if *attacker == source => {
            Fired::about(Target::Object(*blocker))
        }
        _ => None,
    }
}

/// "that creature gets -1/-1 until end of turn" (CR 608.2, 611.2): an effect of this ability's source.
fn shrink_blocker(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(blocker) = item.subject.object() else { return Ok(()) };
    cantrip_mechanics::pump(g, item.object, blocker, -1, -1);
    Ok(())
}

/// "Artifact creatures you control have menace and "Whenever a creature blocks this creature, that creature gets -1/-1
/// until end of turn."" (CR 604.1, 613.1f): one layer 6 grant, so each artifact creature its controller controls has both
/// the keyword and the ability, judged on what layers 1-5 made it.
fn menace_and_trigger(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_menace(q, source, chars);
    statics::grant_delayed_ability(q, source, chars);
}

pub static CARD: CardDef = CardDef {
    delayed: &[triggered(
        when(EventMask::BLOCKS, blocks_this),
        shrink_blocker,
        "Whenever a creature blocks this creature, that creature gets -1/-1 until end of turn.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_artifact_creatures,
        menace_and_trigger,
        TEXT,
    )],
    ..oracle::EXTRACTION_INTEGRANT
};
