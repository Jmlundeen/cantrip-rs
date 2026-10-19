use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_UPKEEP;
use cantrip_mechanics::{harness, targets};
use cantrip_oracle as oracle;

/// "Return target creature card from your graveyard to the battlefield" (CR 400.3, 110.2a): the card returns under its
/// owner's control, who is the player whose graveyard it was in. Nothing happens if the target is no longer legal
/// (CR 608.2b).
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(card).owner;
    g.put_onto_battlefield(card, owner, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::B), "{T}: Add {B}."),
        harness::ability(
            cost("{6}{B}, {T}").and(exile_permanents(1, targets::filters::creature)),
            "{6}{B}, {T}, Exile a creature you control: Harness The Soul Stone.",
        ),
    ],
    delayed: &[triggered(YOUR_UPKEEP, reanimate, "At the beginning of your upkeep, return target creature card from your graveyard to the battlefield.")
        .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD])],
    statics: &[harness::infinity(
        "\u{221e} \u{2014} At the beginning of your upkeep, return target creature card from your graveyard to the battlefield.",
    )],
    ..oracle::THE_SOUL_STONE
};
