use cantrip_api::*;
use cantrip_mechanics::{effects, library, select, sunburst};
use cantrip_oracle as oracle;

/// "A creature, instant, or sorcery card" (CR 205.2a): a card with any of the three types.
fn creature_instant_or_sorcery(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        chars.is(CardType::Creature) || chars.is(CardType::Instant) || chars.is(CardType::Sorcery)
    })
}

/// "Converge - Search your library for a creature, instant, or sorcery card with mana value less than or equal to the
/// number of colors of mana spent to cast this spell, exile that card, then shuffle. You may cast that card without paying
/// its mana cost." The colors are counted as the spell resolves (CR 207.2c; colorless is not a color, CR 105.1). The search
/// may find nothing (CR 701.23b) and the shuffle happens either way. The card is cast as part of the resolution or stays
/// exiled (CR 608.2g, and the ruling); X is 0 and mandatory additional costs are paid (CR 118.9, 107.3b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let colors = sunburst::colors_spent(g, item.object) as u32;
    let rule = select::cards(select::up_to(1))
        .with(select::part(&(creature_instant_or_sorcery as SelectFn), 0, ""))
        .with(select::mana_value_at_most(colors))
        .called("creature, instant, or sorcery");
    let found = library::search(g, you, you, item.object, SearchDestination::Exile, rule)?;
    let exiled: Vec<ObjectId> = found.into_iter().map(|card| g.exile(card)).collect();
    g.shuffle(you);
    g.end_sentence();
    if let Some(&card) = exiled.first() {
        effects::may_cast_without_paying(g, you, card)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::BRING_TO_LIGHT
};
