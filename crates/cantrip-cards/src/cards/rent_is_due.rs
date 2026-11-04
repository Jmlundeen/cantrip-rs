use cantrip_api::*;
use cantrip_mechanics::select::{self, Part, Selection};
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::{choose, effects};
use cantrip_oracle as oracle;

fn creature_or_treasure(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |id| q.chars(id).is(CardType::Creature) || q.chars(id).has_subtype(Subtype::Treasure))
}

/// "Creatures and/or Treasures" (CR 205.3g): a creature, a Treasure artifact, or both (a Treasure creature).
const CREATURE_OR_TREASURE: Part = select::part(&(creature_or_treasure as SelectFn), 0, "");

/// "You may tap two untapped creatures and/or Treasures you control; if you do, draw a card, otherwise sacrifice this" (CR 608.2c-d,
/// 701.26a): the tapping is chosen as the ability resolves, not paid as a cost, so summoning-sick creatures and Treasures may be
/// tapped for it. With fewer than two untapped, or if the player declines, nothing is tapped and this is sacrificed.
fn tap_two_or_sacrifice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let untapped: Vec<ObjectId> = g
        .permanents_of(you)
        .into_iter()
        .filter(|&id| !g.obj(id).tapped && (g.chars(id).is(CardType::Creature) || g.chars(id).has_subtype(Subtype::Treasure)))
        .collect();
    if untapped.len() >= 2 && effects::ask_may(g, you, item.object, Purpose::TapToPay)? {
        let chosen =
            choose::select(g, you, item.object, Purpose::Tap, untapped, Selection::of(select::exactly(2), &[CREATURE_OR_TREASURE]).called("creature or Treasure"))?;
        g.tap_together(&chosen);
        return effects::draw(g, item, 1);
    }
    effects::sacrifice_self(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        tap_two_or_sacrifice,
        "At the beginning of your end step, you may tap two untapped creatures and/or Treasures you control. If you do, draw a card. Otherwise, sacrifice this enchantment.",
    )],
    ..oracle::RENT_IS_DUE
};
