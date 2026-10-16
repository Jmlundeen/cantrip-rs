use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// You may put a creature card from your hand onto the battlefield. It gains haste (no duration is stated, so for as long as
/// it is that object, CR 611.2a). Sacrifice it at the beginning of
/// the next end step. Nobody is asked with no creature card in hand.
fn put_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    let Some(&card) = choose::cards_from(g, player, item.object, Purpose::PutOntoBattlefield, hand, eligible, 1, true)?.first() else { return Ok(()) };
    let returned = g.put_onto_battlefield(card, player, false)?;
    if !g.is_on_battlefield(returned) {
        return Ok(());
    }
    g.add_effect(item.object, returned, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::Indefinite);
    cantrip_mechanics::delayed::at_next_end_step(g, item, 0, Target::Object(returned));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{R}"), put_creature, "{R}: You may put a creature card from your hand onto the battlefield. That creature gains haste. Sacrifice the creature at the beginning of the next end step.")],
    delayed: &[delayed_trigger(cantrip_mechanics::delayed::sacrifice_subject, "Sacrifice the creature.")],
    ..oracle::SNEAK_ATTACK
};
