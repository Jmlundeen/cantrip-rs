use cantrip_api::*;
use cantrip_mechanics::mana::TAP_FOR_ANY_COLOR;
use cantrip_oracle as oracle;

fn creatures_you_control(g: &dyn Game, player: PlayerId) -> Vec<ObjectId> {
    g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect()
}

/// I, II: "Until your next turn, creatures you control gain '{T}: Add one mana of any color.'" The creatures are the
/// ones their controller has as the chapter resolves, fixed then (CR 611.2c); the grant outlives the Saga.
fn grant_mana(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = g.obj(item.object).card;
    for id in creatures_you_control(g, item.controller) {
        g.add_effect(item.object, id, EffectKind::AddAbility(AbilityRef::delayed(me, 0)), Duration::UntilNextTurnOf(item.controller));
    }
    Ok(())
}

/// III: a +1/+1 counter on each creature you control, which gain vigilance, trample and indestructible until end of
/// turn.
fn rally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = creatures_you_control(g, item.controller);
    for &id in &creatures {
        g.add_counters(id, CounterKind::PlusOne, 1);
    }
    let keywords = Keywords::of(&[Keyword::Vigilance, Keyword::Trample, Keyword::Indestructible]);
    for id in creatures {
        g.add_effect(item.object, id, EffectKind::AddKeywords(keywords), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "I \u{2014} Until your next turn, creatures you control gain \"{T}: Add one mana of any color.\"",
            chapter: 1,
            effect: grant_mana,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "II \u{2014} Until your next turn, creatures you control gain \"{T}: Add one mana of any color.\"",
            chapter: 2,
            effect: grant_mana,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "III \u{2014} Put a +1/+1 counter on each creature you control. Those creatures gain vigilance, trample, and indestructible until end of turn.",
            chapter: 3,
            effect: rally,
            ..TriggeredDef::BLANK
        }),
    ],
    // The ability chapters I and II grant.
    delayed: &[TAP_FOR_ANY_COLOR],
    ..oracle::SONG_OF_FREYALISE
};
