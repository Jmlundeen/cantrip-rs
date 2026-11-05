use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Create a 1/1 red Officer creature token" (CR 111.1, 111.4).
fn create_officer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OFFICER, 1)?;
    Ok(())
}

/// "{G}, Exile this card from your graveyard: Create a 1/1 red Officer creature token. Activate only as a sorcery." An
/// activated ability that functions only in its owner's graveyard (CR 113.6m), `from` says so; exiling the card is a cost
/// (CR 602.2), so the card stays in exile even if the ability is countered (CR 701.6a). Reach is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{G}").and(EXILE_THIS),
        create_officer,
        "{G}, Exile this card from your graveyard: Create a 1/1 red Officer creature token. Activate only as a sorcery.",
    )
    .from(ZoneKind::Graveyard)
    .sorcery_speed()],
    ..oracle::PHOTOSYNTHETIC_PHYSICIAN
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
