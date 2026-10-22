use cantrip_api::*;
use cantrip_mechanics::effects::{gain_hexproof_and_indestructible_until_end_of_turn as protect, gain_life};
use cantrip_mechanics::{targets, tiered};
use cantrip_oracle as oracle;

/// "Target permanent gains hexproof and indestructible until end of turn." Nothing happens if the target is no longer
/// legal (CR 608.2b).
fn cure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        protect(g, item.object, permanent);
    }
    Ok(())
}

/// Cure, "You gain 3 life." The spell does not resolve at all with its one target illegal (CR 608.2b), so the life is
/// never gained then.
fn cura(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cure(g, item)?;
    gain_life(g, item, 3)
}

/// "Permanents you control gain hexproof and indestructible until end of turn. You gain 6 life." The permanents are the
/// ones there are as this resolves, and only those (CR 611.2c).
fn curaga(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let permanents = g.permanents_of(item.controller);
    instruction(g, |g| {
        for permanent in permanents {
            protect(g, item.object, permanent);
        }
    });
    g.end_sentence();
    gain_life(g, item, 6)
}

const MODES: &[ModeDef] = &[
    mode(cure, "Cure — {0} — Target permanent gains hexproof and indestructible until end of turn.")
        .targets(&[targets::PERMANENT])
        .cost(mana("{0}")),
    mode(cura, "Cura — {1} — Target permanent gains hexproof and indestructible until end of turn. You gain 3 life.")
        .targets(&[targets::PERMANENT])
        .cost(mana("{1}")),
    mode(curaga, "Curaga — {3}{W} — Permanents you control gain hexproof and indestructible until end of turn. You gain 6 life.")
        .cost(mana("{3}{W}")),
];

static MODAL: ModalDef = tiered::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::RESTORATION_MAGIC };
