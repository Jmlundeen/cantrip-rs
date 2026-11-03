use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::ATTACHED_PERMANENT_BECOMES_TAPPED};
use cantrip_oracle as oracle;

/// "When enchanted land becomes tapped, destroy it." (CR 608.2, 701.8a).
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let objects_0 = item.subject.object().into_iter().collect::<Vec<ObjectId>>();
    g.destroy_all(&objects_0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            ATTACHED_PERMANENT_BECOMES_TAPPED,
            destroy,
            "When enchanted land becomes tapped, destroy it.",
        ),
    ],
    enchant: Some(targets::LAND),
    ..oracle::BLIGHT
};
