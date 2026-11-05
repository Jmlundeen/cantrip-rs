use cantrip_api::*;
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_mechanics::{effects, targets, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "Target creature you control with power 4 or greater" (CR 115.1, 115.2): its power as the layers make it, judged when the
/// target is chosen and again as the ability resolves (CR 608.2b).
fn creature_you_control_with_power_four_or_greater(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.controller(id) == asker.controller && q.chars(id).is(CardType::Creature) && q.chars(id).power.is_some_and(|power| power >= 4))
}

/// "Until end of turn, target creature you control with power 4 or greater gains trample and 'Whenever this creature deals
/// combat damage to a player, draw a card'" (CR 611.2a, 611.2c, 613.1f): both fixed to the creature as the ability resolves; the
/// trigger is the creature's own, so its controller draws.
fn grant_trample_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let this = g.card_id(&CARD);
    // One instruction (`Game::one_instruction`): "gains trample and ..." is a single grant.
    g.one_instruction(&mut |g| {
        trample_until_end_of_turn(g, item.object, creature);
        g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
        Ok(())
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        // CR 106.6: the restriction limits what the mana can be spent on, and the creature spell it pays for may be any.
        mana_ability(TAP, add_any_color(1).only_for(ManaRestriction::CREATURE_SPELLS), "{T}: Add one mana of any color. Spend this mana only to cast a creature spell."),
        ability(TAP, grant_trample_and_draw, "{T}: Until end of turn, target creature you control with power 4 or greater gains trample and \"Whenever this creature deals combat damage to a player, draw a card.\"")
            .targets(&[target(creature_you_control_with_power_four_or_greater)]),
    ],
    delayed: &[triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, |g, item| effects::draw(g, item, 1), "Whenever this creature deals combat damage to a player, draw a card.")],
    ..oracle::HERD_HEIRLOOM
};
