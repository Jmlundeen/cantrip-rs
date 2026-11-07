use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// What the damage may take from any recipient, read before the damage is dealt: each player's life total and each
/// planeswalker's loyalty, since the damage may be redirected (CR 614.9) to one the spell never named, and may not take all of
/// it (CR 120.3b, 120.3c).
struct Before {
    lives: Vec<i32>,
    loyalties: Vec<(ObjectId, i32)>,
}

impl Before {
    fn read(g: &dyn Game) -> Before {
        let lives = g.state().players.iter().map(|player| player.life).collect();
        let loyalties = g
            .state()
            .battlefield()
            .iter()
            .filter(|&&id| g.chars(id).types.has(CardType::Planeswalker))
            .map(|&id| (id, g.obj(id).counters(CounterKind::Loyalty)))
            .collect();
        Before { lives, loyalties }
    }

    /// The most life the damage may gain, from `recipient`: a player's life total or a planeswalker's loyalty as they were
    /// before the damage, a creature's toughness as it is (not reduced by damage marked on it, ruling 2004-10-04); a creature
    /// that is also a planeswalker is under both.
    fn cap(&self, g: &dyn Game, recipient: Target) -> i64 {
        let Target::Object(id) = recipient else {
            return recipient.player().map_or(0, |player| self.lives.get(player.0 as usize).map_or(0, |&life| i64::from(life)));
        };
        let mut cap = i64::MAX;
        if g.chars(id).types.has(CardType::Creature) {
            cap = cap.min(i64::from(g.chars(id).toughness.unwrap_or(0)));
        }
        if let Some(&(_, loyalty)) = self.loyalties.iter().find(|&&(walker, _)| walker == id) {
            cap = cap.min(i64::from(loyalty));
        }
        cap
    }
}

/// "Drain Life deals X damage to any target. You gain life equal to the damage dealt, but not more life than the player's
/// life total before the damage was dealt, the planeswalker's loyalty before the damage was dealt, or the creature's
/// toughness." The damage dealt is what the damage procedure reports after prevention and replacement (CR 120.3), so a
/// prevented or reduced hit gains less. The cap is read before the damage, since the damage may remove life or loyalty (CR
/// 120.3), from the recipient the damage reached, which a replacement may have changed (CR 614.9).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let before = Before::read(g);
    if let Some(result) = g.damage_result(item.object, item.target(0), u32::from(item.x)) {
        let limit = before.cap(g, result.target);
        g.gain_life(item.controller, i64::from(result.amount).min(limit.max(0)) as i32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::ANY], effect), x_mana: XMana::only(Mana::B), ..oracle::DRAIN_LIFE };
