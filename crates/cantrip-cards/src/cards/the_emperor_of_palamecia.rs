use cantrip_api::*;
use cantrip_mechanics::effects::damage_each_opponent;
use cantrip_mechanics::triggers::{self, THIS_ATTACKS};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on [this creature]. Then if it has three or more +1/+1 counters on it, transform it." (CR 122.1, 701.27):
/// the counter first, then the count read, so the third counter is what turns it over. Nothing happens to it once it has left
/// the battlefield (CR 400.7), and it does not transform if it has transformed since the ability was put on the stack (CR 701.27f).
fn counter_then_transform(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    if g.obj(item.object).counters(CounterKind::PlusOne) >= 3 {
        g.transform_by(item.object, item)?;
    }
    Ok(())
}

/// Starfall: "it deals X damage to each opponent, where X is the number of noncreature, nonland cards in your graveyard" (CR 120.3,
/// 404.1): cards only, in its controller's graveyard, counted once as the ability resolves (ruling), tokens not among them.
fn starfall(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g
        .state()
        .zone(Zone::graveyard(item.controller))
        .iter()
        .filter(|&&card| {
            let chars = g.chars(card);
            !chars.is(CardType::Creature) && !chars.is(CardType::Land)
        })
        .count();
    damage_each_opponent(g, item, x as u32);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        starfall,
        "Starfall \u{2014} Whenever The Lord Master of Hell attacks, it deals X damage to each opponent, where X is the number of noncreature, nonland cards in your graveyard.",
    )],
    ..oracle::THE_LORD_MASTER_OF_HELL
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_one_of(&[Color::Blue, Color::Red]).only_for(ManaRestriction::NONCREATURE_SPELLS),
            "{T}: Add {U} or {R}. Spend this mana only to cast a noncreature spell.",
        ),
        triggered(
            when_asked(EventMask::SPELL_CAST, triggers::noncreature_spell_you_cast_with_at_least_mana_spent::<4>),
            counter_then_transform,
            "Whenever you cast a noncreature spell, if at least four mana was spent to cast it, put a +1/+1 counter on The Emperor of Palamecia. Then if it has three or more +1/+1 counters on it, transform it.",
        ),
    ],
    back: Some(&BACK),
    ..oracle::THE_EMPEROR_OF_PALAMECIA
};
