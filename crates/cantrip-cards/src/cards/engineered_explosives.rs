use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// Destroy each nonland permanent with mana value equal to the number of charge counters on this artifact, together.
/// The counters are read off the source even though it was sacrificed to pay for the ability (CR 608.2h).
fn destroy_matching(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let counters = g.obj(item.object).counters(CounterKind::Charge).max(0) as u32;
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| !g.chars(id).is(CardType::Land) && g.mana_value(id) == counters)
        .collect();
    g.destroy_all(&doomed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::sunburst),
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{2}, Sacrifice Engineered Explosives: Destroy each nonland permanent with mana value equal to the number of charge counters on Engineered Explosives.",
        cost: Cost { mana: mana(2, 0, 0, 0, 0, 0), sacrifice_self: true, ..Cost::NONE },
        effect: destroy_matching,
        ..ActivatedDef::BLANK
    })],
    ..oracle::ENGINEERED_EXPLOSIVES
};
