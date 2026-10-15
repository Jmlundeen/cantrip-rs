use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Artifact spells and colorless spells" (CR 601.3, 105.2c): an artifact, or a card with no color.
fn artifact_or_colorless(q: &dyn Query, card: ObjectId) -> bool {
    let c = q.chars(card);
    c.is(CardType::Artifact) || c.colors.is_empty()
}

/// {T}, Pay 1 life: Exile the top card of your library.
fn exile_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(&top) = g.state().zone(Zone::library(item.controller)).first() {
        g.exile(top);
    }
    Ok(())
}

// "You may look at the top card of your library any time" (CR 401.2) is what a client shows its controller.
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}, Pay 1 life: Exile the top card of your library.",
        cost: Cost { tap: true, life: 1, ..Cost::NONE },
        effect: exile_top,
        ..ActivatedDef::BLANK
    })],
    play_rules: &[PlayRule::LibraryTopSpells(artifact_or_colorless)],
    ..oracle::MYSTIC_FORGE
};
