use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "It gains double strike until end of turn" (CR 702.4a, 611.2a): the creature the trigger is about, the lone attacker.
fn gain_double_strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::DoubleStrike);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::CREATURE_YOU_CONTROL_ATTACKS_ALONE,
            gain_double_strike,
            "Rough Divide — Whenever a creature you control attacks alone, it gains double strike until end of turn.",
        ),
        triggered(
            triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER,
            effects::return_first_target_to_battlefield,
            "Whenever Squall deals combat damage to a player, return target permanent card with mana value 3 or less from your graveyard to the battlefield.",
        )
        .targets(&[targets::PERMANENT_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD]),
    ],
    ..oracle::SQUALL_SEED_MERCENARY
};
