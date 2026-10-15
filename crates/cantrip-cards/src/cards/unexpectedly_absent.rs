use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Put target nonland permanent into its owner's library just beneath the top X cards of that library (a token simply
/// ceases to exist).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(id) = targets::object(g, item, 0) {
        g.put_into_library(&[id], LibraryPlace::FromTop(item.x as usize));
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::NONLAND_PERMANENT], effect }),
    ..oracle::UNEXPECTEDLY_ABSENT
};
