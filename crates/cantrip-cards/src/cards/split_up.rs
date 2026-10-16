use cantrip_api::*;
use cantrip_oracle as oracle;

/// Destroys together (CR 701.8a, 101.4) every creature that is tapped (or, with `tapped` false, untapped) as the spell
/// resolves; regeneration and indestructible are the verb's business.
fn destroy_creatures(g: &mut dyn Game, tapped: bool) {
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.obj(id).tapped == tapped)
        .collect();
    g.destroy_all(&doomed);
}

/// Destroy all tapped creatures.
fn destroy_tapped(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    destroy_creatures(g, true);
    Ok(())
}

/// Destroy all untapped creatures.
fn destroy_untapped(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    destroy_creatures(g, false);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy_tapped, "Destroy all tapped creatures."),
    mode(destroy_untapped, "Destroy all untapped creatures."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SPLIT_UP };
