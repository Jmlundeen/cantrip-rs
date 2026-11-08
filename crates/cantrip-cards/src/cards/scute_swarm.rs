use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{lands_you_control, tokens};
use cantrip_oracle as oracle;

/// "Landfall — Whenever a land you control enters, create a 1/1 green Insect creature token. If you control six or more
/// lands, create a token that's a copy of this creature instead." (CR 603.2, 707.2, 608.2c): the lands are counted as the
/// ability resolves, the land that entered among them; below six lands one plain Insect is made, at six or more a single
/// copy of Scute Swarm itself, never both.
fn landfall_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if lands_you_control(g, item.controller) >= 6 {
        let values = g.copiable_values(item.object);
        g.create_token_copies_of(values, item.controller, 1)?;
    } else {
        tokens::create(g, item.controller, &tokens::INSECT, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        LANDFALL,
        landfall_effect,
        "Landfall \u{2014} Whenever a land you control enters, create a 1/1 green Insect creature token. If you control six or more lands, create a token that's a copy of this creature instead.",
    )],
    ..oracle::SCUTE_SWARM
};
