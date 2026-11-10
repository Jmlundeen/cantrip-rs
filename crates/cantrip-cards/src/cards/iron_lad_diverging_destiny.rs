use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Reveal the top card of your library. If it's an artifact card, draw a card." (CR 701.20a, 121.1):
/// `effects::reveal_top_and_draw_if_artifact`, the same reveal-and-draw B'Elanna Torres, Scrapper prints with a may.
fn reveal_and_draw_an_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::reveal_top_and_draw_if_artifact(g, item, None)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        reveal_and_draw_an_artifact,
        "{T}: Reveal the top card of your library. If it's an artifact card, draw a card.",
    )],
    // "You may look at the top card of your library any time" (CR 401.2), which the host shows its controller.
    play_rules: &[PlayRule::LookAtLibraryTop],
    ..oracle::IRON_LAD_DIVERGING_DESTINY
};
