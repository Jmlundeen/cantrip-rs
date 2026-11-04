use cantrip_api::*;
use cantrip_mechanics::{effects, hand, triggers};
use cantrip_oracle as oracle;

/// "You may put an artifact or creature card from your hand onto the battlefield. That permanent gains haste, 'When this permanent
/// leaves the battlefield, it deals damage equal to its mana value to each creature,' and 'At the beginning of your end step,
/// sacrifice this permanent.'" (CR 611.2a, 613.1f, 113.1a): no duration is stated, so the three abilities last for as long as it is
/// that object (CR 400.7). They are gained together, one instruction (CR 608.2c). Nobody is asked with no such card in hand.
fn put_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(put) = hand::may_put_onto_battlefield(g, item.controller, item.object, |c| c.is(CardType::Artifact) || c.is(CardType::Creature))? else {
        return Ok(());
    };
    if !g.is_on_battlefield(put) {
        return Ok(());
    }
    let o = g.obj(item.object);
    let this = o.copied_card().unwrap_or(o.card);
    instruction(g, |g| {
        effects::gain_haste(g, item.object, put);
        g.add_effect(item.object, put, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::Indefinite);
        g.add_effect(item.object, put, EffectKind::AddAbility(AbilityRef::delayed(this, 1)), Duration::Indefinite);
    });
    Ok(())
}

/// "It deals damage equal to its mana value to each creature": the mana value it last had on the battlefield (CR 603.10a, 608.2h;
/// the card's ruling), an {X} in its cost counting as 0 there (CR 202.3e); 0 deals no damage (CR 120.8).
fn damage_equal_to_its_mana_value(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let value = g.state().last_known(item.object).map_or(0, |known| known.mana_value);
    effects::damage_each_creature(g, item, value as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], put_and_gain),
    delayed: &[
        triggered(triggers::THIS_LEAVES, damage_equal_to_its_mana_value, "When this permanent leaves the battlefield, it deals damage equal to its mana value to each creature."),
        triggered(triggers::YOUR_END_STEP, effects::sacrifice_self, "At the beginning of your end step, sacrifice this permanent."),
    ],
    ..oracle::TERMINAL_VELOCITY
};
