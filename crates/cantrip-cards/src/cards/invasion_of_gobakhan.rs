use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Look at target opponent's hand. You may exile a nonland card from it. For as long as that card remains exiled, its
/// owner may play it. A spell cast this way costs {2} more to cast." Looking at someone else's hand is worth doing even
/// with nothing eligible, so the choice is asked of any hand that is not empty.
fn take_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.targets[0] else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    if hand.is_empty() {
        return Ok(());
    }
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| !g.chars(c).is(CardType::Land)).collect();
    let ask = RequestKind::ChooseCards { source: item.object, purpose: Purpose::Exile, cards: hand, eligible, count: 1, up_to: true, kind: "" };
    let Answer::Cards(chosen) = g.ask(item.controller, ask)? else { return Ok(()) };
    let Some(&card) = chosen.first() else { return Ok(()) };
    let exiled = g.exile(card);
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        // Its owner may play it for as long as it stays there (CR 400.7), the spell costing {2} more (CR 601.2f).
        let owner = g.obj(exiled).owner;
        g.allow_play(exiled, owner, PlayGrant { lands: true, more: 2, ..PlayGrant::CAST });
    }
    Ok(())
}

// --- Lightshield Array --------------------------------------------------------------------------------------------

/// "Put a +1/+1 counter on each creature that attacked this turn": whoever controls it now, each one still the object
/// that attacked (CR 400.7) and still a creature on the battlefield.
fn counters_on_attackers(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let attackers: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.attacked_this_turn(id) && g.chars(id).is(CardType::Creature))
        .collect();
    for id in attackers {
        g.add_counters(id, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// "Creatures you control gain hexproof and indestructible until end of turn."
fn protect_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let keywords = Keywords::of(&[Keyword::Hexproof, Keyword::Indestructible]);
    for id in creatures {
        g.add_effect(item.object, id, EffectKind::AddKeywords(keywords), Duration::EndOfTurn);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::YOUR_END_STEP, counters_on_attackers, "At the beginning of your end step, put a +1/+1 counter on each creature that attacked this turn."),
        ability(SACRIFICE_THIS, protect_creatures, "Sacrifice this enchantment: Creatures you control gain hexproof and indestructible until end of turn."),
    ],
    ..oracle::LIGHTSHIELD_ARRAY
};

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, take_a_card, "When this Siege enters, look at target opponent's hand. You may exile a nonland card from it. For as long as that card remains exiled, its owner may play it. A spell cast this way costs {2} more to cast.")
        .targets(&[targets::OPPONENT])],
    back: Some(&BACK),
    ..oracle::INVASION_OF_GOBAKHAN
};
