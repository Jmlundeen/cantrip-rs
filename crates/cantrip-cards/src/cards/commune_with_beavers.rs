use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top_select, Rest};
use cantrip_mechanics::select::{self, Part};
use cantrip_oracle as oracle;

/// "An artifact, creature, or land card" (CR 205.2a): a card with any of the three types, as it is in the library.
fn artifact_creature_or_land(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        chars.is(CardType::Artifact) || chars.is(CardType::Creature) || chars.is(CardType::Land)
    })
}

const ARTIFACT_CREATURE_OR_LAND: Part = select::part(&(artifact_creature_or_land as SelectFn), 0, "");

/// "Look at the top three cards of your library. You may reveal an artifact, creature, or land card from among them and put it
/// into your hand. Put the rest on the bottom of your library in any order." (CR 401.4, 701.20a)
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::cards(select::up_to(1)).with(ARTIFACT_CREATURE_OR_LAND).called("artifact, creature, or land");
    look_at_top_select(g, item.controller, item.object, 3, rule, true, Rest::BottomChosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], look), ..oracle::COMMUNE_WITH_BEAVERS };
