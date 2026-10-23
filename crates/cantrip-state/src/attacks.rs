//! Judging a declaration of attackers (CR 508.1a-d): one validator for every restriction, requirement and cost a
//! declaration answers to, from the rules the question holds (`AttackRules`, beside the attackers that may attack, what they
//! may attack and what attacks cost). `AttackRules::judge` says whether a declaration may be made as it is
//! (`AttackVerdict::complete`), why not (`AttackVerdict::issue`), what it costs, and which attacks may be added to it on the
//! way to one that may (`AttackVerdict::addable`). The core checks every answer with it, the scenario harness reads it, and
//! the host gives its verdict to a client as its player declares (`CheckAttacks`), so no client works CR 508.1d out.
//!
//! CR 508.1d counts requirements: the declaration must obey as many as any declaration that obeys every restriction can
//! without paying a cost. A goaded creature (CR 701.15b) is under two for each player who goaded it - attack, and attack a
//! player other than that one - and "attacks that player this combat if able" is one: attacking either of two players may
//! obey as many as the other, and both are legal. `solve` is the one search; every restriction the engine has is on one
//! attacker at a time (who may attack, and what), so the most is each attacker's best attack, added up.

use crate::{AttackCost, ManaCost, ObjectId, PlayerId, Target, count_u32};

/// What a requirement on a declaration of attackers asks of its attacker (CR 508.1d).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttackAim {
    /// That it attack, whatever it attacks: "attacks each combat if able", a goad's first requirement (CR 701.15b).
    Anything,
    /// That it attack this player, planeswalker or battle: "attacks that player this combat if able".
    At(Target),
    /// That it attack a player other than this one: a goad's second requirement, the goader's (CR 701.15b).
    PlayerOtherThan(PlayerId),
}

/// One requirement on a declaration of attackers (CR 508.1d), one for each effect, ability or player it comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequiredAttack {
    pub attacker: ObjectId,
    pub aim: AttackAim,
}

/// The rules of a declaration of attackers past which attacks are allowed (CR 508.1c-d), held in the question so the
/// game-log tree moves through them: the requirements; the most of them any declaration that obeys every restriction obeys
/// without paying a cost, which the declaration must obey too; and one such declaration, the engine's suggestion.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AttackRules {
    pub required: Vec<RequiredAttack>,
    pub most: u32,
    pub suggested: Vec<(ObjectId, Target)>,
}

/// What the rules make of a declaration of attackers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AttackVerdict {
    /// It may be declared as it is (CR 508.1a-d); its cost, if any, is paid once it is declared (CR 508.1h-j).
    pub complete: bool,
    /// The attacks, by creatures not attacking, that some legal declaration extending it has (empty unless asked for).
    pub addable: Vec<(ObjectId, Target)>,
    /// Why it may not be declared as it is; none when it may.
    pub issue: Option<AttackIssue>,
    /// What declaring it costs (CR 508.1h), zero when nothing.
    pub cost: ManaCost,
    /// Nothing is required of the declaration (the most is 0): any declaration of `addable` attacks, each creature attacking
    /// once, is legal, so a client need not ask about each one.
    pub free: bool,
}

/// Why a declaration of attackers may not be made as it is, as a caption reads it: `creature`, then `words`, then `other`
/// where there is one ("Ursine Monstrosity" "must attack" Bot2; "Grizzly Bears" "must attack a player other than" Bot2), with
/// the rule it breaks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttackIssue {
    pub creature: ObjectId,
    pub words: String,
    pub other: Option<Target>,
    pub rule: &'static str,
}

impl AttackRules {
    /// Whether `attacker` attacking `at` obeys `r`.
    pub fn obeys(r: &RequiredAttack, attacker: ObjectId, at: Target) -> bool {
        r.attacker == attacker
            && match r.aim {
                AttackAim::Anything => true,
                AttackAim::At(t) => t == at,
                AttackAim::PlayerOtherThan(p) => matches!(at, Target::Player(q) if q != p),
            }
    }

    fn gain(&self, attacker: ObjectId, at: Target) -> u32 {
        count_u32(self.required.iter().filter(|r| Self::obeys(r, attacker, at)).count())
    }

    /// How many of the requirements a declaration obeys (CR 508.1d).
    pub fn obeyed(&self, attacks: &[(ObjectId, Target)]) -> u32 {
        attacks.iter().map(|&(a, t)| self.gain(a, t)).sum()
    }

    /// The best `attacker` can obey: the most any one attack of it among `targets` obeys (and not attacking obeys none),
    /// with one such attack; with `free_only`, among attacks that cost nothing (CR 508.1d).
    fn best(&self, attacker: ObjectId, targets: &[Target], costs: &[AttackCost], free_only: bool) -> (u32, Option<Target>) {
        let mut best = (0, None);
        for &t in targets {
            if free_only && !AttackCost::free(costs, attacker, t) {
                continue;
            }
            let g = self.gain(attacker, t);
            if g > best.0 {
                best = (g, Some(t));
            }
        }
        best
    }

    /// The most requirements any declaration of `eligible` attacking among `targets` obeys without paying a cost (CR
    /// 508.1d), with one that does: each attacker's best free attack.
    pub fn solve(&self, eligible: &[ObjectId], targets: &[Target], costs: &[AttackCost]) -> (u32, Vec<(ObjectId, Target)>) {
        let mut most = 0;
        let mut suggested = Vec::new();
        for &a in eligible {
            if let (g, Some(t)) = self.best(a, targets, costs, true) {
                most += g;
                suggested.push((a, t));
            }
        }
        (most, suggested)
    }

    /// What the rules make of `attacks`, a declaration of `eligible` creatures attacking among `targets`, with what attacks
    /// cost (CR 508.1a-d); with `addable`, which attacks may be added on the way to a legal one too.
    pub fn judge(&self, eligible: &[ObjectId], targets: &[Target], costs: &[AttackCost], attacks: &[(ObjectId, Target)], addable: bool) -> AttackVerdict {
        let issue = |creature, words: &str, other, rule| Some(AttackIssue { creature, words: words.to_string(), other, rule });
        let mut found = None;
        let mut mendable = true;
        for (i, &(a, t)) in attacks.iter().enumerate() {
            if attacks[..i].iter().any(|x| x.0 == a) {
                found = issue(a, "can attack only once", None, "508.1a");
            } else if !eligible.contains(&a) || !targets.contains(&t) {
                found = issue(a, "can't attack", Some(t), "508.1c");
            }
            if found.is_some() {
                mendable = false;
                break;
            }
        }
        if found.is_none() && self.obeyed(attacks) < self.most {
            // An attacker obeying fewer than it could, and a requirement its best attack obeys that its attack does not.
            found = eligible.iter().find_map(|&a| {
                let now = attacks.iter().find(|x| x.0 == a).map(|x| x.1);
                let (best, at) = self.best(a, targets, costs, true);
                let have = now.map_or(0, |t| self.gain(a, t));
                let at = at?;
                (have < best).then(|| {
                    let r = self.required.iter().find(|r| Self::obeys(r, a, at) && now.is_none_or(|t| !Self::obeys(r, a, t)))?;
                    match r.aim {
                        AttackAim::Anything => issue(a, "must attack", None, "508.1d"),
                        AttackAim::At(t) => issue(a, "must attack", Some(t), "508.1d"),
                        AttackAim::PlayerOtherThan(p) => issue(a, "must attack a player other than", Some(Target::Player(p)), "508.1d"),
                    }
                })?
            });
        }
        let complete = found.is_none();
        let addable = if addable && mendable {
            // Every restriction is on one attacker at a time: an attack may be added where the others not yet attacking can
            // make up the rest at their best, paying if need be.
            let declared = self.obeyed(attacks);
            let free: Vec<ObjectId> = eligible.iter().copied().filter(|a| !attacks.iter().any(|x| x.0 == *a)).collect();
            let rest: Vec<(ObjectId, u32)> = free.iter().map(|&a| (a, self.best(a, targets, costs, false).0)).collect();
            let all: u32 = rest.iter().map(|r| r.1).sum();
            let mut list = Vec::new();
            for &(a, best) in &rest {
                for &t in targets {
                    if declared + self.gain(a, t) + all - best >= self.most {
                        list.push((a, t));
                    }
                }
            }
            list
        } else {
            Vec::new()
        };
        AttackVerdict { complete, addable, issue: found, cost: AttackCost::total(costs, attacks), free: self.most == 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: ObjectId = ObjectId(5);
    const P2: PlayerId = PlayerId(1);
    const P3: PlayerId = PlayerId(2);

    /// Goaded by P2 and told to attack P2 this combat (three players, Ursine Monstrosity): attacking P2 obeys two of three
    /// (attack, and attack P2), attacking P3 two (attack, and attack a player other than P2); both are legal, and not
    /// attacking is refused.
    #[test]
    fn a_goaded_creature_told_to_attack_its_goader_may_attack_either_player() {
        let targets = [Target::Player(P2), Target::Player(P3)];
        let mut r = AttackRules {
            required: vec![
                RequiredAttack { attacker: U, aim: AttackAim::Anything },
                RequiredAttack { attacker: U, aim: AttackAim::PlayerOtherThan(P2) },
                RequiredAttack { attacker: U, aim: AttackAim::At(Target::Player(P2)) },
            ],
            ..Default::default()
        };
        (r.most, r.suggested) = r.solve(&[U], &targets, &[]);
        assert_eq!(r.most, 2);
        assert!(r.judge(&[U], &targets, &[], &[(U, Target::Player(P2))], false).complete);
        assert!(r.judge(&[U], &targets, &[], &[(U, Target::Player(P3))], false).complete);
        let none = r.judge(&[U], &targets, &[], &[], true);
        assert!(!none.complete && none.addable.len() == 2);
        assert_eq!(none.issue.map(|i| i.rule), Some("508.1d"));
    }

    /// A cost to attack is never paid to obey a requirement (CR 508.1d): with every attack costing something, nothing is
    /// required, and attacking and paying is still legal.
    #[test]
    fn a_requirement_is_never_obeyed_by_paying() {
        let targets = [Target::Player(P2)];
        let costs = [AttackCost { attacker: U, target: Target::Player(P2), cost: ManaCost { generic: 2, ..ManaCost::ZERO }, source: ObjectId(9) }];
        let mut r = AttackRules { required: vec![RequiredAttack { attacker: U, aim: AttackAim::Anything }], ..Default::default() };
        (r.most, r.suggested) = r.solve(&[U], &targets, &costs);
        assert_eq!(r.most, 0);
        assert!(r.judge(&[U], &targets, &costs, &[], false).complete);
        let paid = r.judge(&[U], &targets, &costs, &[(U, Target::Player(P2))], false);
        assert!(paid.complete && paid.cost.generic == 2);
    }
}
