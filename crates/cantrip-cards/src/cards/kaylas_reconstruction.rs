use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::select::{self, ARTIFACT_OR_CREATURE};
use cantrip_oracle as oracle;

/// "Look at the top seven cards of your library. Put up to X artifact and/or creature cards with mana value 3 or less from
/// among them onto the battlefield. Put the rest on the bottom of your library in a random order." Each card is judged alone
/// for "mana value 3 or less" (CR 202.3e; the ruling: it is not a total).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::Selection::new(select::up_to(u32::from(item.x))).with(ARTIFACT_OR_CREATURE).with(select::mana_value_at_most(3)).called("artifact or creature");
    library::look_at_top_onto_battlefield_select(g, item.controller, item.object, 7, rule, false, library::Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::KAYLAS_RECONSTRUCTION
};
