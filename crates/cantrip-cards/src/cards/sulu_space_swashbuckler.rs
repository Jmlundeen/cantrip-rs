use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "If an Equipment or Spacecraft entered the battlefield under your control this turn" (CR 603.4): an intervening "if",
/// asked as the ability triggers and again as it resolves, read of the history, which keeps each permanent as it entered
/// (CR 603.6a): its controller and its subtypes then, whatever it has become since, wherever it has gone and whoever
/// controls it now. The subtypes are read through Sulu's words (CR 612.2).
fn equipment_or_spacecraft_entered(q: &dyn Query, asker: &Asker) -> bool {
    let equipment = q.words(asker.source).subtype(Subtype::Equipment);
    let spacecraft = q.words(asker.source).subtype(Subtype::Spacecraft);
    q.history().entered().any(|e| {
        let chars = e.chars(q);
        e.controller == asker.controller && (chars.has_subtype(equipment) || chars.has_subtype(spacecraft))
    })
}

/// "Whenever Sulu becomes tapped, if an Equipment or Spacecraft entered the battlefield under your control this turn, draw
/// a card." (CR 603.2e, 603.4.)
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_BECOMES_TAPPED,
        draw_a_card,
        "Whenever Sulu becomes tapped, if an Equipment or Spacecraft entered the battlefield under your control this turn, draw a card.",
    )
    .condition(equipment_or_spacecraft_entered)],
    ..oracle::SULU_SPACE_SWASHBUCKLER
};
