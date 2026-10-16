use cantrip_api::*;
use cantrip_mechanics::{mayhem, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Target creature card with mana value 3 or less from your graveyard."
fn cheap_creature_card(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Creature], 3)
}

/// Return the creature card to the battlefield under the ability's controller. It gains "This creature attacks each
/// combat if able" and the sacrifice trigger, for as long as that object stays (CR 611.2a, 400.7).
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let returned = g.put_onto_battlefield(card, item.controller, false)?;
    if !g.is_on_battlefield(returned) {
        return Ok(());
    }
    g.add_effect(item.object, returned, EffectKind::MustAttack, Duration::Indefinite);
    let this = g.obj(item.object).card;
    g.add_effect(item.object, returned, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, reanimate, "When Carnage enters, return target creature card with mana value 3 or less from your graveyard to the battlefield. It gains \"This creature attacks each combat if able\" and \"When this creature deals combat damage to a player, sacrifice it.\"")
        .targets(&[target(cheap_creature_card)])],
    ways: &[mayhem::way(mana("{B}{R}"), "Mayhem {B}{R}")],
    delayed: &[triggered(cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER, cantrip_mechanics::effects::sacrifice_self, "When this creature deals combat damage to a player, sacrifice it.")],
    ..oracle::CARNAGE_CRIMSON_CHAOS
};
