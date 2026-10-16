use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "You may put a creature card from your hand onto the battlefield. That creature gains haste. Sacrifice that creature
/// at the beginning of the next end step." Up to one creature card; nothing is asked when the hand has none. The haste
/// has no end but the object (CR 611.2a), and the delayed sacrifice names the object (CR 603.7c), so a creature that
/// has since become a new object (CR 400.7) is left alone. When it is spliced onto an Arcane spell (CR 702.47) `item`
/// is that spell, so the delayed ability is this card's own, not the item's.
fn put_creature_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    let Some(&card) = choose::cards_from(g, player, item.object, Purpose::PutOntoBattlefield, hand, eligible, 1, true)?.first() else { return Ok(()) };
    let returned = g.put_onto_battlefield(card, player, false)?;
    if !is_live(g, returned) || !g.is_on_battlefield(returned) {
        return Ok(());
    }
    g.add_effect(item.object, returned, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::Indefinite);
    let this = g.card_id(&CARD);
    g.delayed_trigger(
        DelayedWhen::Step { step: Step::End, on_turn_of: None },
        item.object,
        player,
        AbilityRef::delayed(this, 0),
        Target::Object(returned),
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], put_creature_in),
    splice: Some(mana("{2}{R}{R}")),
    delayed: &[delayed_trigger(cantrip_mechanics::delayed::sacrifice_subject, "Sacrifice that creature at the beginning of the next end step.")],
    ..oracle::THROUGH_THE_BREACH
};
