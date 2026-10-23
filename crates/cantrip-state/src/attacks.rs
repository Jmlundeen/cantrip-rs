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
//! obey as many as the other, and both are legal. `solve` is the one search: every restriction on one attacker at a time
//! (who may attack, and what) makes the most each attacker's best attack, added up, and a cap on how many may attack one
//! target (CR 508.1c) is filled in that same order, so the suggestion never asks for one creature more than a target allows.

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
/// game-log tree moves through them: the requirements; the most creatures that may attack each player, planeswalker or
/// battle, where something caps it; the most of them any declaration that obeys every restriction obeys without paying a
/// cost, which the declaration must obey too; and one such declaration, the engine's suggestion.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AttackRules {
    pub required: Vec<RequiredAttack>,
    /// The most creatures that may attack each capped player, planeswalker or battle (CR 508.1c), one entry each.
    pub limits: Vec<(Target, u32)>,
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
    /// Nothing is required of the declaration (the most is 0) and nothing caps how many may attack a target: any
    /// declaration of `addable` attacks, each creature attacking once, is legal, so a client need not ask about each one.
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

    /// How many creatures may attack `at` (CR 508.1c), or `None` when nothing caps it.
    fn limit(&self, at: Target) -> Option<u32> {
        self.limits.iter().find(|&&(t, _)| t == at).map(|&(_, n)| n)
    }

    /// Whether `at`'s cap leaves room for one more attack among `attacks`.
    fn room(&self, at: Target, attacks: &[(ObjectId, Target)]) -> bool {
        self.limit(at).is_none_or(|n| count_u32(attacks.iter().filter(|x| x.1 == at).count()) < n)
    }

    /// How many of the requirements a declaration obeys (CR 508.1d).
    pub fn obeyed(&self, attacks: &[(ObjectId, Target)]) -> u32 {
        attacks.iter().map(|&(a, t)| self.gain(a, t)).sum()
    }

    /// The best `attacker` can obey: the most any one attack of it among `targets` obeys (and not attacking obeys none),
    /// with one such attack; with `free_only`, among attacks that cost nothing (CR 508.1d); passing over an attack that
    /// `placed` already fills the cap of (CR 508.1c).
    fn best(&self, attacker: ObjectId, targets: &[Target], costs: &[AttackCost], free_only: bool, placed: &[(ObjectId, Target)]) -> (u32, Option<Target>) {
        let mut best = (0, None);
        for &t in targets {
            if free_only && !AttackCost::free(costs, attacker, t) {
                continue;
            }
            if !self.room(t, placed) {
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
        let mut suggested: Vec<(ObjectId, Target)> = Vec::new();
        for &a in eligible {
            // shortcut: a capped target is filled in eligible order, which can fall short of the most when two creatures'
            // requirements compete for one capped player; make this an exact assignment if such a card is printed.
            if let (g, Some(t)) = self.best(a, targets, costs, true, &suggested) {
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
        // CR 508.1c: more creatures declared attacking one target than its cap allows is illegal, though an effect may
        // still put another there once they are declared. Nothing extends a declaration already over a cap, so none is
        // addable.
        if found.is_none() {
            for &(t, n) in &self.limits {
                let mut attacking = attacks.iter().filter(|x| x.1 == t).map(|x| x.0);
                if let Some(creature) = attacking.nth(n as usize) {
                    found = issue(creature, "can't attack", Some(t), "508.1c");
                    mendable = false;
                    break;
                }
            }
        }
        if found.is_none() && self.obeyed(attacks) < self.most {
            // An attacker obeying fewer than it could, and a requirement its best attack obeys that its attack does not.
            found = eligible
                .iter()
                .find_map(|&a| {
                    let now = attacks.iter().find(|x| x.0 == a).map(|x| x.1);
                    let placed: Vec<(ObjectId, Target)> = attacks.iter().copied().filter(|x| x.0 != a).collect();
                    let (best, at) = self.best(a, targets, costs, true, &placed);
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
                })
                // Fewer obeyed than the most always leaves some attacker short of its best; should none be found, the
                // declaration is still refused, never complete by accident.
                .or_else(|| issue(attacks.first().map_or(ObjectId::NONE, |x| x.0), "must attack", None, "508.1d"));
        }
        let complete = found.is_none();
        let addable = if addable && mendable {
            // Each restriction is on one attacker at a time, and a cap is on how many attack one target (CR 508.1c): an
            // attack may be added where that target has room and the others not yet attacking can make up the rest at
            // their best, paying if need be.
            let declared = self.obeyed(attacks);
            let free: Vec<ObjectId> = eligible.iter().copied().filter(|a| !attacks.iter().any(|x| x.0 == *a)).collect();
            let mut list = Vec::new();
            for &a in &free {
                for &t in targets {
                    if !self.room(t, attacks) {
                        continue;
                    }
                    let mut placed = attacks.to_vec();
                    placed.push((a, t));
                    let others: u32 = free.iter().filter(|&&b| b != a).map(|&b| self.best(b, targets, costs, false, &placed).0).sum();
                    if declared + self.gain(a, t) + others >= self.most {
                        list.push((a, t));
                    }
                }
            }
            list
        } else {
            Vec::new()
        };
        // Two attacks of `addable` may still be one too many for a capped target (CR 508.1c), so a capped declaration is
        // never free of asking.
        AttackVerdict { complete, addable, issue: found, cost: AttackCost::total(costs, attacks), free: self.most == 0 && self.limits.is_empty() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: ObjectId = ObjectId(5);
    const V: ObjectId = ObjectId(6);
    const P2: PlayerId = PlayerId(1);
    const P3: PlayerId = PlayerId(2);
    /// A planeswalker of P2 that one creature at most may attack (Tomik, Orzhov Lawmage).
    const W: Target = Target::Object(ObjectId(7));

    fn capped(at: Target) -> AttackRules {
        AttackRules { limits: vec![(at, 1)], ..Default::default() }
    }

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

    /// CR 508.1c: a second creature declared attacking a planeswalker no more than one may attack makes the declaration
    /// illegal, and the issue is the caption's.
    #[test]
    fn a_second_attacker_on_a_capped_target_is_illegal() {
        let targets = [Target::Player(P2), W];
        let verdict = capped(W).judge(&[U, V], &targets, &[], &[(U, W), (V, W)], false);
        assert!(!verdict.complete);
        let issue = verdict.issue.expect("an illegal declaration says why");
        assert_eq!((issue.creature, issue.words.as_str(), issue.other, issue.rule), (V, "can't attack", Some(W), "508.1c"));
    }

    /// One creature attacking the capped planeswalker, or another target, is legal.
    #[test]
    fn a_capped_target_allows_one_attacker_and_no_cap_on_another() {
        let targets = [Target::Player(P2), W];
        assert!(capped(W).judge(&[U, V], &targets, &[], &[(U, W)], false).complete);
        assert!(capped(W).judge(&[U, V], &targets, &[], &[(U, W), (V, Target::Player(P2))], false).complete);
        assert!(capped(W).judge(&[U, V], &targets, &[], &[(U, Target::Player(P2)), (V, Target::Player(P2))], false).complete);
    }

    /// Two creatures may attack one target where nothing caps it.
    #[test]
    fn without_a_limit_two_may_attack_one_target() {
        let targets = [Target::Player(P2), W];
        assert!(AttackRules::default().judge(&[U, V], &targets, &[], &[(U, W), (V, W)], false).complete);
    }

    /// Nothing is addable onto a target already at its cap, and what is addable elsewhere still is.
    #[test]
    fn addable_never_adds_a_second_attacker_to_a_capped_target() {
        let targets = [Target::Player(P2), W];
        let verdict = capped(W).judge(&[U, V], &targets, &[], &[(U, W)], true);
        assert!(verdict.complete);
        assert!(verdict.addable.contains(&(V, Target::Player(P2))));
        assert!(!verdict.addable.contains(&(V, W)));
        // Two attacks it offers may still be one too many here, so a client has to ask.
        assert!(!verdict.free);
    }

    /// The suggestion fills a capped target no further than the cap, even where a requirement would send another there.
    #[test]
    fn a_suggestion_never_exceeds_a_limit() {
        let targets = [Target::Player(P2), W];
        let mut r = capped(Target::Player(P2));
        r.required = vec![
            RequiredAttack { attacker: U, aim: AttackAim::Anything },
            RequiredAttack { attacker: U, aim: AttackAim::At(Target::Player(P2)) },
            RequiredAttack { attacker: V, aim: AttackAim::Anything },
            RequiredAttack { attacker: V, aim: AttackAim::At(Target::Player(P2)) },
        ];
        (r.most, r.suggested) = r.solve(&[U, V], &targets, &[]);
        assert_eq!(r.suggested.iter().filter(|x| x.1 == Target::Player(P2)).count(), 1);
        assert_eq!(r.most, 3);
        assert!(r.judge(&[U, V], &targets, &[], &r.suggested, false).complete);
    }
}
