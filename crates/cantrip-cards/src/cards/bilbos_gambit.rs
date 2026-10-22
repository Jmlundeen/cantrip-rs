use cantrip_api::*;
use cantrip_mechanics::{gift, targets, tokens};
use cantrip_oracle as oracle;

/// "Return target spell to its owner's hand" (not a counter, so a spell that can't be countered goes too; a copy that
/// leaves the stack ceases to exist, CR 707.10a). "If the gift was promised, players can't cast spells this turn":
/// a rule about each player in the game as it resolves (CR 611.2c), from every zone. The gift itself, a Treasure for
/// the chosen opponent, was given before this runs (CR 702.174j), and a copy of the spell has it promised too
/// (the ruling). A target no longer legal stops the whole spell, gift included (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(spell) = targets::object(g, item, 0) {
        g.remove_spell(spell, CounterTo::Hand);
    }
    if item.gift.is_some() {
        g.end_sentence();
        let players: Vec<PlayerId> = g.state().apnap().collect();
        instruction(g, |g| {
            for player in players {
                g.add_player_effect(item.controller, item.object, EffectKind::CantCastSpellsFrom(player, ZoneKinds::ALL), Duration::EndOfTurn);
            }
        });
    }
    Ok(())
}

/// "Gift a Treasure" (CR 702.174h): the Treasure is the card's own token.
pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    gift: Some(gift::TREASURE),
    ..oracle::BILBOS_GAMBIT
};
