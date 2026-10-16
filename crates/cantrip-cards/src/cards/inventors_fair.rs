use cantrip_api::*;
use cantrip_mechanics::artifacts_you_control;
use cantrip_mechanics::library::{search_to_hand, Find};
use cantrip_oracle as oracle;

/// Whether the controller controls three or more artifacts (CR 109.5).
fn three_artifacts(q: &dyn Query, asker: &Asker) -> bool {
    artifacts_you_control(q, asker.controller) >= 3
}

/// Search your library for an artifact card, reveal it, put it into your hand, then shuffle. The search may come up
/// empty; the shuffle follows either way.
fn tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| q.chars(c).is(CardType::Artifact))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{4}, {T}").and(SACRIFICE_THIS), tutor, "{4}, {T}, Sacrifice Inventors' Fair: Search your library for an artifact card, reveal it, put it into your hand, then shuffle. Activate only if you control three or more artifacts.")
            .condition(three_artifacts),
        triggered(cantrip_mechanics::triggers::YOUR_UPKEEP, |g, item| cantrip_mechanics::effects::gain_life(g, item, 1), "At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life.")
            .condition(three_artifacts),
    ],
    ..oracle::INVENTORS_FAIR
};
