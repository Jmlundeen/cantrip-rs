use cantrip_api::*;
use cantrip_mechanics::{effects, library, targets};
use cantrip_oracle as oracle;

/// "Target artifact or creature's owner puts it on their choice of the top or bottom of their library. Surveil 1." The owner
/// chooses (`library::owner_puts_on_top_or_bottom`), then the caster surveils 1 (CR 701.25), a new sentence (CR 608.2c). With its
/// only target illegal the spell does nothing at all, so there is no surveil then (CR 608.2b).
fn top_or_bottom_then_surveil(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    library::owner_puts_on_top_or_bottom(g, item.object, permanent)?;
    g.end_sentence();
    effects::surveil(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE], top_or_bottom_then_surveil),
    ..oracle::LOST_IN_SPACE
};
