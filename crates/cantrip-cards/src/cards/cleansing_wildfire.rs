use cantrip_api::*;
use cantrip_mechanics::lands::destroy_target_its_controller_may_fetch_basic;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield
/// tapped, then shuffle. Draw a card." If the land is no longer a legal target the spell does not resolve (CR 608.2b), so
/// the draw is not reached either; a legal land that is not destroyed still gives the search and the draw.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_its_controller_may_fetch_basic(g, item, 0, true)?;
    g.end_sentence();
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::LAND], effect), ..oracle::CLEANSING_WILDFIRE };
