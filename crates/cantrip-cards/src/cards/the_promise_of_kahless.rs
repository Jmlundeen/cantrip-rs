use cantrip_api::*;
use cantrip_mechanics::{effects, select};
use cantrip_oracle as oracle;

/// "Klingon or Warrior" (CR 205.3m): either of the two creature types, read through this spell's own words so that a
/// text-changing effect changes them (CR 612.2).
fn klingon_or_warrior(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let klingon = q.words(choice.source).subtype(Subtype::Klingon);
    let warrior = q.words(choice.source).subtype(Subtype::Warrior);
    Verdict::each(choice, |id| {
        let chars = q.chars(id);
        chars.has_subtype(klingon) || chars.has_subtype(warrior)
    })
}

const KLINGON_OR_WARRIOR: select::Part = select::part(&(klingon_or_warrior as SelectFn), 0, "Klingon or Warrior");
const A_KLINGON_OR_WARRIOR: select::Selection =
    select::Selection::of(select::up_to(1), &[KLINGON_OR_WARRIOR]).called("Klingon or Warrior");

/// "You may sacrifice a Klingon or Warrior. If you do, return all other Klingon and Warrior creature cards from your
/// graveyard to the battlefield." (CR 701.21a, 608.2c-d, 608.2f, 612.2): the permanent is chosen as the spell resolves,
/// or none is; only if one was actually sacrificed are the creature cards of those two types in the caster's own
/// graveyard - "other", so not the one just sacrificed, which is read by the new object its move made (CR 400.7) -
/// returned, together, under the caster's control (CR 110.2a). A card of either type in another player's graveyard is
/// not looked at.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let sacrificed = effects::may_sacrifice(g, you, item.object, A_KLINGON_OR_WARRIOR)?;
    if sacrificed.is_empty() {
        return Ok(());
    }
    let klingon = g.words(item.object).subtype(Subtype::Klingon);
    let warrior = g.words(item.object).subtype(Subtype::Warrior);
    let cards: Vec<(ObjectId, PlayerId)> = g
        .state()
        .zone(Zone::graveyard(you))
        .to_vec()
        .into_iter()
        .filter(|&card| {
            let chars = g.chars(card);
            chars.is(CardType::Creature)
                && (chars.has_subtype(klingon) || chars.has_subtype(warrior))
                && !sacrificed.objects().contains(&card)
        })
        .map(|card| (card, you))
        .collect();
    g.put_all_onto_battlefield(&cards)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::THE_PROMISE_OF_KAHLESS };
