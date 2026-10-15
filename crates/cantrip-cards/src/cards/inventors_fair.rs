use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_mechanics::library::{search_to_hand, Find};
use cantrip_oracle as oracle;

/// Whether the controller controls three or more artifacts (CR 109.5).
fn three_artifacts(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().filter(|&id| q.chars(id).is(CardType::Artifact)).count() >= 3
}

/// Search your library for an artifact card, reveal it, put it into your hand, then shuffle. The search may come up
/// empty; the shuffle follows either way.
fn tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| q.chars(c).is(CardType::Artifact))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{4}, {T}, Sacrifice Inventors' Fair: Search your library for an artifact card, reveal it, put it into your hand, then shuffle. Activate only if you control three or more artifacts.",
            cost: Cost { mana: mana(4, 0, 0, 0, 0, 0), tap: true, sacrifice_self: true, ..Cost::NONE },
            effect: tutor,
            condition: Some(three_artifacts),
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life.",
            listens: EventMask::STEP_BEGAN,
            matches: cantrip_mechanics::triggers::your_upkeep,
            effect: cantrip_mechanics::effects::gain_life::<1>,
            condition: Some(three_artifacts),
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::INVENTORS_FAIR
};
