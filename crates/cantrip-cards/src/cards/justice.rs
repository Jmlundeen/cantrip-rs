use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever a red creature or spell deals damage" (CR 603.2, 109.2): a red source that is a spell on the stack or a
/// creature permanent on the battlefield; a creature card in another zone (Trumpeting Carnosaur's discard) is not a creature.
fn red_creature_or_spell_deals_damage(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let Event::Damage { source, .. } = event else { return None };
    let source = *source;
    let zone = q.state().obj(source).zone;
    let is_source = zone.is(ZoneKind::Stack)
        || (zone.is(ZoneKind::Battlefield) && q.chars(source).is(CardType::Creature));
    if is_source && q.chars(source).colors.has(q.words(asker.source).color(Color::Red)) {
        return Some(Fired { subject: Target::Object(source), data: [0; 2] });
    }
    None
}

/// "This enchantment deals that much damage to that creature's or spell's controller." (CR 608.2, 120.3).
fn enchantment_deals_that_much_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let amount = item.matched_damage_amount();
    let recipients: Vec<Target> = {
        let q: &dyn Query = g;
        item.subject.object().map(|o: ObjectId| Target::Player(q.controller(o))).into_iter().collect()
    };
    let batch: Vec<(ObjectId, Target, u32)> = recipients.iter().map(|t| (item.object, *t, amount)).collect();
    g.damage_together(&batch);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            |g, item| effects::sacrifice_this_unless_pay(g, item, cost("{W}{W}")),
            "At the beginning of your upkeep, sacrifice this enchantment unless you pay {W}{W}.",
        ),
        triggered(
            when_asked(EventMask::DAMAGE, red_creature_or_spell_deals_damage),
            enchantment_deals_that_much_damage,
            "Whenever a red creature or spell deals damage, this enchantment deals that much damage to that creature's or spell's controller.",
        )
        .damage_amount()
        .sums_per_subject(),
    ],
    ..oracle::JUSTICE
};
