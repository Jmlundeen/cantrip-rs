use cantrip_api::*;
use cantrip_mechanics::library::look_at_top_take_one_rest_to_graveyard;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Look at that many cards from the top of your library. Put one of them into your hand and the rest into your graveyard" (CR 401.2):
/// the controller of the creature, which may be another creature that gained the ability. A library with fewer cards shows what it has.
fn look_at_that_many(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top_take_one_rest_to_graveyard(g, item.controller, item.object, item.data[0] as usize)
}

/// "Put a +1/+1 counter on target creature you control. It gains this card's other abilities" (CR 122.1, 613.1f): the combat damage
/// trigger (not Find New Host itself, which is not "other"), from this card's `delayed` list. With no duration, for as long as it stays on the
/// battlefield (CR 611.2a). A creature that is no longer a legal target gets neither (CR 608.2b).
fn counter_and_gain_abilities(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let this = g.card_id(&CARD);
    g.add_counters(creature, CounterKind::PlusOne, 1);
    instruction(g, |g| {
        g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::Indefinite);
    });
    Ok(())
}

const COMBAT_DAMAGE: &str = "Whenever this creature deals combat damage to a player, look at that many cards from the top of your library. Put one of them into your hand and the rest into your graveyard.";
const FIND_NEW_HOST: &str = "Find New Host — {2}{U/B}, Exile this card from your graveyard: Put a +1/+1 counter on target creature you control. It gains this card's other abilities. Activate only as a sorcery.";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::DAMAGE, triggers::deals_combat_damage_to_player_subject), look_at_that_many, COMBAT_DAMAGE),
        ability(cost("{2}{U/B}").and(EXILE_THIS), counter_and_gain_abilities, FIND_NEW_HOST)
            .targets(&[targets::CREATURE_YOU_CONTROL])
            .from(ZoneKind::Graveyard)
            .sorcery_speed(),
    ],
    // The abilities Find New Host gives: this card's other abilities.
    delayed: &[
        triggered(when(EventMask::DAMAGE, triggers::deals_combat_damage_to_player_subject), look_at_that_many, COMBAT_DAMAGE),
    ],
    ..oracle::SYMBIOTE_SPIDER_MAN
};
