use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "The owner of target nonland permanent puts it into their library second from the top or on the bottom." Convoke is the core's,
/// read from the card's JSON.
fn put_into_library(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    effects::owner_puts_into_library_second_from_top_or_on_the_bottom(g, item, permanent)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], put_into_library),
    labels: &["Second from the top", "On the bottom"],
    ..oracle::TEMPORAL_CLEANSING
};
