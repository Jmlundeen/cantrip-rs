use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn golem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::GOLEM, 1)?;
    Ok(())
}

/// Put a +1/+1 counter on each creature token you control; they gain trample until end of turn. The tokens are
/// the ones there as the ability resolves (CR 611.2c).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let tokens: Vec<ObjectId> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| g.obj(id).token && g.chars(id).is(CardType::Creature))
        .collect();
    for &token in &tokens {
        g.add_counters(token, CounterKind::PlusOne, 1);
    }
    for &token in &tokens {
        g.add_effect(item.object, token, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Sandstorm Salvager enters, create a 3/3 colorless Golem artifact creature token.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: golem,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{2}, {T}: Put a +1/+1 counter on each creature token you control. They gain trample until end of turn.",
            cost: Cost { mana: mana(2, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: grow,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::SANDSTORM_SALVAGER
};

pub static EXTRA: &[&CardDef] = &[&tokens::GOLEM];
